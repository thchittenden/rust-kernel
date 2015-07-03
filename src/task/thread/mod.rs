//!
//! This module contains definitions of the `Thread` struct which keeps track of all information
//! pertaining to a thread of execution.
//!
//! The Thread structure has only one requirement in terms of layout and that is that the
//! `stack_bottom` field must be at offset 0x10 from the thread base. This is required due to the
//! way stack overflow checks are performed. Because LLVM provides no way of customizing how these
//! checks are generated and hard codes them in depending on the target operating system, we are
//! required to use one of its pre-coded layouts. %fs:0x10 was the most reasonable choice and
//! appears in the DragonFly operating system. 
//!
//! LLVM should really support custom targets!
//!
use alloc::boxed::Box;
use core::prelude::*;
use core::atomic::{AtomicIsize, ATOMIC_ISIZE_INIT, Ordering};
use core::mem;
use collections::link::{DoubleLink, HasDoubleLink};
use util::{KernResult, asm};
logger_init!(Trace);

/// There is some "wiggle room" in stack checking which allows the stack to go slightly beyond
/// whatever stack_bottom is set to. This is a problem if stack_bottom is contained in that area
/// because it may get overwritten! We thus allocate a small redzone between stack_bottom and the
/// actual end of the stack.
const REDZONE_SIZE: usize = 16;
const STACK_SIZE: usize = 2048;
const STACK_TOP:  usize = STACK_SIZE - 1;
const ARG_OFFSET: usize = STACK_SIZE - 2;
const RET_OFFSET: usize = STACK_SIZE - 4;
const EBP_OFFSET: usize = STACK_SIZE - 5;
const EBX_OFFSET: usize = STACK_SIZE - 6;
const EDI_OFFSET: usize = STACK_SIZE - 7;
const ESI_OFFSET: usize = STACK_SIZE - 8;
static NEXT_TID: AtomicIsize = ATOMIC_ISIZE_INIT;

/// The entry point for all new threads. Currently this doesn't do much.
extern fn thread_entry(thread: &Thread) -> ! {
    trace!("starting thread {}", thread.tid);

    // Since we came from a context switch we need to reenable interrupts.
    asm::enable_interrupts();

    // Run the thread.
    thread.run()
}

#[repr(C, packed)]
pub struct Thread {
    pub tid: i32,
    pub pid: i32,
    stack_cur: usize, 
    stack_top: usize,
    stack_bottom: usize, // This MUST be at offset 0x10
    sched_node: DoubleLink<Thread>,
    threadfn: fn() -> !,
    stack: [usize; STACK_SIZE]
}

impl Thread {

    pub fn new(f: fn() -> !) -> KernResult<Box<Thread>> {
        Box::emplace(|thread: &mut Thread| {
            thread.tid = NEXT_TID.fetch_add(1, Ordering::Relaxed) as i32;
            thread.pid = 0;
            thread.sched_node = Default::default();
            thread.threadfn = f;
            thread.stack_cur = &thread.stack[ESI_OFFSET] as *const usize as usize;
            thread.stack_top = &thread.stack[STACK_TOP] as *const usize as usize;
            thread.stack_bottom = &thread.stack[REDZONE_SIZE] as *const usize as usize;
            thread.stack[ARG_OFFSET] = thread as *const Thread as usize;
            thread.stack[RET_OFFSET] = unsafe { mem::transmute(thread_entry) };
            thread.stack[EBP_OFFSET] = thread.stack_top;
            thread.stack[EBX_OFFSET] = 0;
            thread.stack[EDI_OFFSET] = 0;
            thread.stack[ESI_OFFSET] = 0;
        })
    }

    pub fn run(&self) -> ! {
        (self.threadfn)()
    }

}

impl HasDoubleLink<Thread> for Thread {
    fn dlink(&self) -> &DoubleLink<Thread> {
        &self.sched_node
    }
    fn dlink_mut(&mut self) -> &mut DoubleLink<Thread> {
        &mut self.sched_node
    }
}
