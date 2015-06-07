//!
//! This module contains the definition of a linked list that uses embedded nodes.
//! 
//! In the linked list, the next pointer always points towards the tail and the previous pointer
//! always points to towards the head.
//!
use core::prelude::*;
use util::raw::Raw;
use util::link::HasDoubleLink;
use util::Pointer;

/// A queue.
pub struct LinkedList<T: HasDoubleLink<T=T, P=P>, P: Pointer<To=T>> {
    pub len: usize,
    pub head: Option<P>,
    pub tail: Option<Raw<T>>
}

/// Creates a static empty queue.
#[macro_export]
macro_rules! static_linkedlist {
    () => ({
        LinkedList {
            len: 0,
            head: None,
            tail: None
        }
    });
}

impl<T: HasDoubleLink<T=T, P=P>, P: Pointer<To=T> + HasDoubleLink<T=T,P=P>> LinkedList<T, P> {
   
    /// Creates a new empty queue.
    pub fn new() -> LinkedList<T, P> {
        static_linkedlist!()
    }

    pub fn push_head(&mut self, mut new_head: P) {
        assert!(new_head.dlink().next.link.is_none());
        assert!(new_head.dlink().prev.is_none());
        assert!(self.head.is_none() == self.tail.is_none());
        match self.head.take() {
            None => {
                self.tail = Some(unsafe { Raw::new(new_head.as_mut()) });
                self.head = Some(new_head);
            }
            Some(mut head) => {
                head.dlink_mut().prev = Some(unsafe { Raw::new(new_head.as_mut()) });
                new_head.dlink_mut().next.link = Some(head);
                self.head = Some(new_head);
            }
        }
        self.len += 1;
    }

    
    pub fn push_tail(&mut self, mut new_tail: P) {
        assert!(new_tail.dlink().next.link.is_none());
        assert!(new_tail.dlink().prev.is_none());
        assert!(self.head.is_none() == self.tail.is_none());
        match self.tail.take() {
            None => {
                self.tail = Some(unsafe { Raw::new(new_tail.as_mut()) });
                self.head = Some(new_tail);
            }
            Some(mut tail) => {
                new_tail.dlink_mut().prev = Some(tail.clone());
                self.tail = Some(unsafe { Raw::new(new_tail.as_mut()) });
                tail.dlink_mut().next.link = Some(new_tail);
            }
        }
        self.len += 1;
    }

    pub fn pop_head(&mut self) -> Option<P> {
        assert!(self.head.is_none() == self.tail.is_none());
        self.head.take().map(|mut head| {
            match head.dlink_mut().next.link.take() {
                None => {
                    // List is now empty.
                    self.tail = None;
                }
                Some(mut new_head) => {
                    // List not empty.
                    new_head.dlink_mut().prev = None;
                    self.head = Some(new_head);
                }
            }
            self.len -= 1;
            assert!(self.head.is_none() == self.tail.is_none());
            assert!(head.dlink().next.link.is_none());
            assert!(head.dlink().prev.is_none());
            head
        })
    }

    pub fn pop_tail(&mut self) -> Option<P> {
        assert!(self.head.is_none() == self.tail.is_none());
        self.tail.take().map(|mut tail| {
            let tail = match tail.dlink_mut().prev.take() {
                None => {
                    // List is now empty.
                    self.head.take().unwrap()
                }
                Some(mut new_tail) => {
                    let res = new_tail.dlink_mut().next.link.take().unwrap();
                    self.tail = Some(new_tail);
                    res
                }
            };
            self.len -= 1;
            assert!(self.head.is_none() == self.tail.is_none());
            assert!(tail.dlink().next.link.is_none());
            assert!(tail.dlink().prev.is_none());
            tail
        })
    }

    pub fn borrow_tail(&self) -> Option<&T> {
        self.tail.as_ref().map(|tail| &**tail)
    }

    pub fn borrow_tail_mut(&mut self) -> Option<&mut T> {
        self.tail.as_mut().map(|tail| &mut**tail)
    }

    pub fn length(&self) -> usize {
        self.len
    }

}

impl<T: HasDoubleLink<T=T, P=P>, P: Pointer<To=T>> Default for LinkedList<T, P> {
    fn default() -> LinkedList<T, P> {
        static_linkedlist!()
    }
}
