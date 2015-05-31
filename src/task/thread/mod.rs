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
use core::prelude::*;
use core::atomic::{AtomicIsize, ATOMIC_ISIZE_INIT, Ordering};
use core::mem;
use alloc::boxed::Box;
use collections::node::{Node, HasNode};
use util::asm;
logger_init!(Trace);

const STACK_SIZE: usize = 1017;
const STACK_TOP:  usize = STACK_SIZE - 1;
const ARG_OFFSET: usize = STACK_SIZE - 2;
const RET_OFFSET: usize = STACK_SIZE - 4;
const EBP_OFFSET: usize = STACK_SIZE - 5;
const EBX_OFFSET: usize = STACK_SIZE - 6;
const EDI_OFFSET: usize = STACK_SIZE - 7;
const ESI_OFFSET: usize = STACK_SIZE - 8;
static next_tid: AtomicIsize = ATOMIC_ISIZE_INIT;

extern fn thread_entry(tcb: &Thread) {
    asm::enable_interrupts();
    loop { 
        trace!("in thread {}", tcb.tid);
    }
}

#[repr(C, packed)]
pub struct Thread {
    tid: i32,
    pid: i32,
    stack_cur: usize, 
    stack_top: usize,
    stack_bottom: usize, // This MUST be at offset 0x10
    sched_node: Node<Thread>,
    stack: [usize; STACK_SIZE]
}

impl Thread {

    pub fn new<F>(f: F) -> Option<Box<Thread>> where F: Fn() -> () {
        Box::emplace(|thread: &mut Thread| {
            thread.tid = next_tid.fetch_add(1, Ordering::Relaxed) as i32;
            thread.pid = 0;
            thread.sched_node = Node { next: None, prev: None };
            thread.stack_cur = &thread.stack[ESI_OFFSET] as *const usize as usize;
            thread.stack_top = &thread.stack[STACK_TOP] as *const usize as usize;
            thread.stack_bottom = &thread.stack[0] as *const usize as usize;
            thread.stack[ARG_OFFSET] = thread as *const Thread as usize;
            thread.stack[RET_OFFSET] = unsafe { mem::transmute(thread_entry) };
            thread.stack[EBP_OFFSET] = thread.stack_top;
            thread.stack[EBX_OFFSET] = 0;
            thread.stack[EDI_OFFSET] = 0;
            thread.stack[ESI_OFFSET] = 0;
        })
    }

}

impl HasNode<Thread> for Thread {
    fn node(&self) -> &Node<Thread> {
        &self.sched_node
    }
    fn node_mut(&mut self) -> &mut Node<Thread> {
        &mut self.sched_node
    }
}