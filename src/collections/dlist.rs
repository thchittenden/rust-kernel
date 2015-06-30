//!
//! This module contains the definition of a linked list that uses embedded nodes.
//! 
//! In the linked list, the next pointer always points towards the tail and the previous pointer
//! always points to towards the head.
//!
use alloc::boxed::Box;
use core::prelude::*;
use core::ops::DerefMut;
use super::raw::Raw;
use super::link::HasDoubleLink;

/// A doubly linked list.
pub struct DList<T: HasDoubleLink<T>> {
    len: usize,
    head: Option<Box<T>>,
    tail: Option<Raw<T>>
}

impl<T: HasDoubleLink<T>> DList<T> {
   
    /// Creates a new empty list.
    pub const fn new() -> DList<T> {
        DList {
            len: 0,
            head: None,
            tail: None
        }
    }

    /// Pushes an element to the head of the list.
    pub fn push_head(&mut self, mut new_head: Box<T>) {
        assert!(new_head.dlink().next.link.is_none());
        assert!(new_head.dlink().prev.is_none());
        assert!(self.head.is_none() == self.tail.is_none());
        match self.head.take() {
            None => {
                self.tail = Some(unsafe { Raw::new(new_head.deref_mut()) });
                self.head = Some(new_head);
            }
            Some(mut head) => {
                head.dlink_mut().prev = Some(unsafe { Raw::new(new_head.deref_mut()) });
                new_head.dlink_mut().next.link = Some(head);
                self.head = Some(new_head);
            }
        }
        self.len += 1;
    }

    /// Pushes an element to the tail of the list.
    pub fn push_tail(&mut self, mut new_tail: Box<T>) {
        assert!(new_tail.dlink().next.link.is_none());
        assert!(new_tail.dlink().prev.is_none());
        assert!(self.head.is_none() == self.tail.is_none());
        match self.tail.take() {
            None => {
                self.tail = Some(unsafe { Raw::new(new_tail.deref_mut()) });
                self.head = Some(new_tail);
            }
            Some(mut tail) => {
                new_tail.dlink_mut().prev = Some(tail.clone());
                self.tail = Some(unsafe { Raw::new(new_tail.deref_mut()) });
                tail.dlink_mut().next.link = Some(new_tail);
            }
        }
        self.len += 1;
    }

    /// Tries to remove an element from the head of the list.
    pub fn pop_head(&mut self) -> Option<Box<T>> {
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

    /// Tries to remove an element from the tail of the list.
    pub fn pop_tail(&mut self) -> Option<Box<T>> {
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

    pub fn borrow_head(&self) -> Option<&T> {
        self.head.as_ref().map(|head| &**head)
    }

    pub fn borrow_head_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|head| &mut**head)
    }

    pub fn borrow_tail(&self) -> Option<&T> {
        self.tail.as_ref().map(|tail| &**tail)
    }

    pub fn borrow_tail_mut(&mut self) -> Option<&mut T> {
        self.tail.as_mut().map(|tail| &mut**tail)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

}

impl<T: HasDoubleLink<T>> Default for DList<T> {
    fn default() -> DList<T> {
        DList::new()
    }
}
