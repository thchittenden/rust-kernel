//!
//! This module implements a wrapper around the *mut T type that allows circular references in data
//! structures. 
//!
//! Care must be taken when constructing these that dangling Raw pointers aren't left around
//! because as soon as the Box leaves the collection, it is assumed to be unique!
//!
use core::ops::{Deref, DerefMut};

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


