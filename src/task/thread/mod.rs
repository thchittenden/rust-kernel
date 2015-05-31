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
use core::mem;
use alloc::boxed::Box;
use collections::node::{Node, HasNode};
logger_init!(Trace);

const STACK_SIZE: usize = 1017;
const RET_OFFSET: usize = STACK_SIZE - 1;
const EBP_OFFSET: usize = STACK_SIZE - 2;
const EBX_OFFSET: usize = STACK_SIZE - 3;
const EDI_OFFSET: usize = STACK_SIZE - 4;
const ESI_OFFSET: usize = STACK_SIZE - 5;

fn thread_entry() {
    loop { trace!("in thread") }
}

#[repr(C, packed)]
pub struct Thread {
    tid: i32,
    pid: i32,
    stack_cur: usize, 
    stack_top: usize,
    stack_bottom: usize, // This MUST be at offset 0x10
    sched_node: Node<Thread>,
    stack: [u32; STACK_SIZE]
}

impl Thread {

    pub fn new<F>(f: F) -> Option<Box<Thread>> where F: Fn() -> () {
        Box::emplace(|thread: &mut Thread| {
            thread.tid = 0;
            thread.pid = 0;
            thread.sched_node = Node { next: None, prev: None };
            thread.stack[RET_OFFSET] = unsafe { mem::transmute(thread_entry) };
            thread.stack[EBP_OFFSET] = 0;
            thread.stack[EBX_OFFSET] = 0;
            thread.stack[EDI_OFFSET] = 0;
            thread.stack[ESI_OFFSET] = 0;
            thread.stack_cur = &thread.stack[EDI_OFFSET] as *const u32 as usize;
            thread.stack_top = &thread.stack[STACK_SIZE] as *const u32 as usize;
            thread.stack_bottom = &thread.stack[0] as *const u32 as usize;
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
