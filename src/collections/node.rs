//!
//! This module contains the definition of the `Node` object used by all collections.
//!
//! This `Node` owns the pointer to the next object thus objects can only be in one queue at a
//! time. This is additionally constrained by the fact that HasNode can only return a single Node
//! which further enforces being in only a single queue. 
//!
use core::option::Option;
use core::ops::{Deref, DerefMut};
use alloc::boxed::Box;

pub struct Raw<T> {
    ptr: *mut T
}

impl<T> Raw<T> {
    pub unsafe fn new(t: &mut T) -> Raw<T> {
        Raw { ptr: t as *mut T }
   }
    pub fn clone(&self) -> Raw<T> {
        Raw { ptr: self.ptr }
    }
}

impl<T> Deref for Raw<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Raw<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

pub trait HasNode<T> {
    fn node(&self) -> &Node<T>;
    fn node_mut(&mut self) -> &mut Node<T>;
}

pub struct Node<T> {
    pub next: Option<Box<T>>,
    pub prev: Option<Raw<T>>,
}

/// We can box up any type that has a node. TODO this is not right...
impl<T: HasNode<T>> HasNode<T> for Box<T> {
    fn node(&self) -> &Node<T> {
        (**self).node()
    }
    fn node_mut(&mut self) -> &mut Node<T> {
        (**self).node_mut()
    }
}
