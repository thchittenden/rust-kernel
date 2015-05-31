//!
//! This module contains the definition of a linked list that uses embedded nodes.
//! 
//! In the linked list, the next pointer always points towards the tail and the previous pointer
//! always points to towards the head.
//!
use core::prelude::*;
use alloc::boxed::Box;
use node::{Raw, Node, HasNode};

/// A queue.
pub struct LinkedList<T: HasNode<T>> {
    pub head: Option<Box<T>>,
    pub tail: Option<Raw<T>>
}

/// Creates a static empty queue.
#[macro_export]
macro_rules! static_linkedlist {
    () => ({
        LinkedList {
            head: None,
            tail: None
        }
    });
}

impl<T: HasNode<T>> LinkedList<T> {
   
    /// Creates a new empty queue.
    pub fn new() -> LinkedList<T> {
        static_linkedlist!()
    }

    pub fn push_head(&mut self, mut new_head: Box<T>) {
        assert!(new_head.node().next.is_none());
        assert!(new_head.node().prev.is_none());
        assert!(self.head.is_none() == self.tail.is_none());
        match self.head.take() {
            None => {
                self.tail = Some(unsafe { Raw::new(&mut *new_head) });
                self.head = Some(new_head);
            }
            Some(mut head) => {
                head.node_mut().prev = Some(unsafe { Raw::new(&mut *new_head) });
                new_head.node_mut().next = Some(head);
                self.head = Some(new_head);
            }
        }
    }

    
    pub fn push_tail(&mut self, mut new_tail: Box<T>) {
        assert!(new_tail.node().next.is_none());
        assert!(new_tail.node().prev.is_none());
        assert!(self.head.is_none() == self.tail.is_none());
        match self.tail.take() {
            None => {
                self.tail = Some(unsafe { Raw::new(&mut *new_tail) });
                self.head = Some(new_tail);
            }
            Some(mut tail) => {
                new_tail.node_mut().prev = Some(tail.clone());
                self.tail = Some(unsafe { Raw::new(&mut *new_tail) });
                tail.node_mut().next = Some(new_tail);
            }
        }
    }

    pub fn pop_head(&mut self) -> Option<Box<T>> {
        assert!(self.head.is_none() == self.tail.is_none());
        self.head.take().map(|mut head| {
            match head.node_mut().next.take() {
                None => {
                    // List is now empty.
                    self.tail = None;
                }
                Some(mut new_head) => {
                    // List not empty.
                    new_head.node_mut().prev = None;
                    self.head = Some(new_head);
                }
            }
            assert!(self.head.is_none() == self.tail.is_none());
            assert!(head.node().next.is_none());
            assert!(head.node().prev.is_none());
            head
        })
    }

    pub fn pop_tail(&mut self) -> Option<Box<T>> {
        assert!(self.head.is_none() == self.tail.is_none());
        self.tail.take().map(|mut tail| {
            let tail = match tail.node_mut().prev.take() {
                None => {
                    // List is now empty.
                    self.head.take().unwrap()
                }
                Some(mut new_tail) => {
                    let res = new_tail.node_mut().next.take().unwrap();
                    self.tail = Some(new_tail);
                    res
                }
            };
            assert!(self.head.is_none() == self.tail.is_none());
            assert!(tail.node().next.is_none());
            assert!(tail.node().prev.is_none());
            tail
        })
    }

}
