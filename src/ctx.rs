use crate::arch::TaskContext;
use core::{
    future::Future, mem::MaybeUninit, panic, pin::Pin 
};
extern crate alloc;
use alloc::boxed::Box;
use memory_addr::VirtAddr;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(usize)]
/// The policy of the scheduler
pub enum ContextType {
    /// The default coroutine context
    COROUTINE,
    /// The kernel thread context
    THREAD,
    /// Unknown context
    UNKNOWN,
}

/// The task context combined future and traditional context
pub struct Context {
    /// The task's thread context
    pub thread_ctx: TaskContext,
    /// The task's future context
    pub fut: MaybeUninit<Pin<Box<dyn Future<Output = i32> + 'static + Send>>>,
    /// The context type
    pub ctx_type: ContextType,
}

impl Context {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        Self {
            thread_ctx: TaskContext::new(),
            fut: MaybeUninit::uninit(),
            ctx_type: ContextType::COROUTINE,
        }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        self.thread_ctx.sp = kstack_top.as_usize();
        self.thread_ctx.ra = entry;
        self.thread_ctx.tp = tls_area.as_usize();
    }
    
    /// Initializes the context for a new task, with the given future.
    pub fn init_future<F, T>(&mut self, future: F)
    where
        F: FnOnce() -> T,
        T: Future<Output = i32> + 'static + Send,
    {
        self.fut.write(Box::pin(future()));
    }

    /// Set the context type
    pub fn set_ctx_type(&mut self, ctx_type: ContextType) {
        self.ctx_type = ctx_type;
    }

    pub fn set_kstack_top(&mut self, kstack_top: VirtAddr) {
        self.thread_ctx.sp = kstack_top.as_usize();
    }
}

pub unsafe extern "C" fn switch(prev_ctx: &mut Context, next_ctx: &mut Context, f: impl FnOnce()) {
    let prev_type = prev_ctx.ctx_type;
    let next_type = next_ctx.ctx_type;
    // No matter what the next_ctx is, we should set it to COROUTINE
    match (prev_type, next_type) {
        (ContextType::COROUTINE, ContextType::COROUTINE) => f(),
        (ContextType::COROUTINE, ContextType::THREAD) => crate::restore_context(&mut next_ctx.thread_ctx),
        (ContextType::THREAD, _) => {
            crate::context_switch(&mut prev_ctx.thread_ctx, &mut next_ctx.thread_ctx);
        },
        (_, _) => panic!("Unsupport context switch"),
    }
}
