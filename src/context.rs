use crate::arch::TrapFrame;
use core::{
    mem::MaybeUninit,
    future::Future,
    pin::Pin, 
};
extern crate alloc;
use alloc::boxed::Box;
use memory_addr::VirtAddr;

#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(C)]
/// The policy of the scheduler
pub enum ContextType {
    /// The default coroutine context
    COROUTINE = 0,
    /// The kernel thread context
    THREAD = 1,
    /// The supervisor interrupt context
    STRAP = 2,
    /// The user interrupt context
    UTRAP = 3,
    /// Unknown context
    UNKNOWN,
}

impl From<usize> for ContextType {
    #[inline]
    fn from(value: usize) -> Self {
        match value {
            0 => ContextType::COROUTINE,
            1 => ContextType::THREAD,
            2 => ContextType::STRAP,
            3 => ContextType::UTRAP,
            _ => ContextType::UNKNOWN,
        }
    }
}

impl From<ContextType> for isize {
    #[inline]
    fn from(ctx_type: ContextType) -> Self {
        match ctx_type {
            ContextType::COROUTINE => 0,
            ContextType::THREAD => 1,
            ContextType::STRAP => 2,
            ContextType::UTRAP => 3,
            ContextType::UNKNOWN => -1,
        }
    }
}

/// The task context combined future and traditional context
pub struct Context {
    /// The task's registers context
    pub trap_frame: *mut TrapFrame,
    /// The task's future context
    pub fut: MaybeUninit<Pin<Box<dyn Future<Output = i32> + 'static + Send>>>,
    /// The context type
    pub ctx_type: ContextType,
}

impl Context {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        Self {
            trap_frame: 0 as *mut TrapFrame,
            fut: MaybeUninit::uninit(),
            ctx_type: ContextType::COROUTINE,
        }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, _entry: usize, _kstack_top: VirtAddr, _tls_area: VirtAddr) {
        // self.regs.sp = kstack_top.as_usize();
        // self.regs.ra = entry;
        // self.regs.tp = tls_area.as_usize();
        // self.ctx_type = ContextType::THREAD;
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

    /// Set trap frame
    pub fn set_trap_frame(&mut self, trap_frame: *mut TrapFrame) {
        self.trap_frame = trap_frame;
    }
}


