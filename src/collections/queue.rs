use core::prelude::*;
use alloc::boxed::Box;
use node::{Node, HasNode};

pub struct Queue<T: HasNode<T>> {
    pub head: Option<Box<T>>,
    pub tail: Option<*const T>
}

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
    
    pub fn new() -> Queue<T> {
        static_queue!()
    }

}
