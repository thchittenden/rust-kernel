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
use collections::node::{Node, HasNode};

#[repr(C, packed)]
pub struct Thread {
    
    tid: i32,
    pid: i32,
    stack_cur: usize, 
    stack_top: usize,
    stack_bottom: usize, // This MUST be at offset 0x10
    sched_node: Node<Thread>

}

impl HasNode<Thread> for Thread {
    fn get_node(&self) -> &Node<Thread> {
        &self.sched_node
    }
    fn get_node_mut(&mut self) -> &mut Node<Thread> {
        &mut self.sched_node
    }
}
