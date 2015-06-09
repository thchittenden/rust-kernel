//!
//! This module implements a wrapper around the *mut T type that allows circular references in data
//! structures. 
//!
//! Care must be taken when constructing these that dangling Raw pointers aren't left around
//! because as soon as the Box leaves the collection, it is assumed to be unique!
//!
use alloc::boxed::Box;
use core::prelude::*;
use core::ops::{Deref, DerefMut};

pub struct Raw<T: ?Sized> {
    ptr: *mut T
}

impl<T: ?Sized> Raw<T> {
    pub unsafe fn new(t: &mut T) -> Raw<T> {
        Raw { ptr: t as *mut T }
    }
    
    pub unsafe fn from_box(b: &mut Box<T>) -> Raw<T> {
        Raw { ptr: &mut **b as *mut T }
    }

    pub fn clone(&self) -> Raw<T> {
        Raw { ptr: self.ptr }
    }

    /// Returns a reference to the contents of the pointer.
    ///
    /// # Safety
    ///
    /// This is unsafe because it returns a reference of an arbitrary lifetime.
    pub unsafe fn as_ref<'a>(&self) -> &'a T {
        &*self.ptr
    }

    /// Returns a mutable reference to the contents of the pointer.
    ///
    /// # Safety
    ///
    /// This is unsafe because it returns a reference of an arbitrary lifetime.
    pub unsafe fn as_mut<'a>(&mut self) -> &'a mut T {
        &mut*self.ptr
    }
}

impl<T: ?Sized> Deref for Raw<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // This is safe because the lifetime of the output is bound by the lifetime of the pointer.
        unsafe { self.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Raw<T> {
    fn deref_mut(&mut self) -> &mut T {
        // This is safe because the lifetime of the output is bound by the lifetime of the pointer.
        unsafe { self.as_mut() }
    }
}


