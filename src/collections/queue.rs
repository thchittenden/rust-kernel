//!
//! This module contains the definition of a simple queue supporting enqueue/dequeue operations.
//!
use core::prelude::*;
use alloc::boxed::Box;
use node::{Node, HasNode};

/// A queue.
pub struct Queue<T: HasNode<T>> {
    pub head: Option<Box<T>>,
    pub tail: Option<*const T>
}

/// Creates a static empty queue.
#[macro_export]
macro_rules! static_queue {
    () => ({
        Queue {
            head: None,
            tail: None
        }
    });
}

impl<T: HasNode<T>> Queue<T> {
   
    /// Creates a new empty queue.
    pub fn new() -> Queue<T> {
        static_queue!()
    }

}
