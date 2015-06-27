//! Kernel box implementation. This was heavily lifted from the std library's Box.
//!
//! An important difference between the std library Box and this box is that we must gracefully
//! handle failure conditions. Thus when constructing Boxes we must return an Option in case the
//! allocation fails.s 
use core::prelude::*;
use core::ptr::Unique;
use core::hash::{self, Hash};
use core::cmp::Ordering;
use core::mem;
use core::fmt;
use core::ops::{Deref, DerefMut, CoerceUnsized};
use core::marker::Unsize;
use core::intrinsics::drop_in_place;
logger_init!(Trace);

/// A pointer type for heap allocations.
pub struct Box<T: ?Sized>(*mut T);

impl<T> Box<T> {
   
    fn make_box(mut uniq: Unique<T>) -> Box<T> {
        Box(unsafe { uniq.get_mut() as *mut T })
    }

    /// Allocates memory on the heap and then moves `x` into it.
    pub fn new(x: T) -> Option<Box<T>> {
        ::allocate(x).map(Self::make_box)
    }

    /// Allocates memory and calls the initialization function on it. This helps avoid copying
    /// large data structures on the stack. This is especially important when allocating stacks!
    pub fn emplace<F>(init: F) -> Option<Box<T>> where F: Fn(&mut T) {
        ::allocate_emplace(init).map(Self::make_box)
    }

}

impl<T: ?Sized> Box<T> {
    
    /// Extracts the raw pointer from the box. 
    ///
    /// # Safety
    ///
    /// This is unsafe because there will be no guarantee that this pointer will be deallocated. 
    pub unsafe fn into_raw(self) -> *mut T {
        // "Forget" ourselves so our destructor is never run.
        let ptr = self.0;
        mem::forget(self);
        ptr
    }

    /// Rebuilds a box from a raw pointer
    ///
    /// # Safety
    ///
    /// This is unsafe because there is no verification that this pointer is unique or that it is
    /// on the heap.
    pub unsafe fn from_raw(ptr: *mut T) -> Box<T> {
        Box(ptr)
    }

}

/// Allow casting from a Box<T> to a Box<U> where T implements U.
impl<T: ?Sized+Unsize<U>, U: ?Sized> CoerceUnsized<Box<U>> for Box<T> {}

impl <T: ?Sized> Drop for Box<T> {
    /// Deallocates the pointer on the heap. We must pay special attention
    /// that we manually drop the contents of the box, otherwise they may
    /// be lost forever.
    fn drop(&mut self) {
        // If we haven't already been dropped, drop the box contents and deallocate the storage.
        if self.0 as *const () as usize != mem::POST_DROP_USIZE {
            trace!("dropping {:p}", self.0 as *const ());
            unsafe { drop_in_place(&mut *self.0) };
            ::deallocate(unsafe { Unique::new(self.0) });
        }
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T { 
        unsafe { &*self.0 } 
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T { 
        unsafe { &mut *self.0 }
    }
}


impl<T: PartialEq> PartialEq for Box<T> {
    #[inline]
    fn eq(&self, other: &Box<T>) -> bool { PartialEq::eq(&**self, &**other) }
    #[inline]
    fn ne(&self, other: &Box<T>) -> bool { PartialEq::ne(&**self, &**other) }
}

impl<T: PartialOrd> PartialOrd for Box<T> {
    #[inline]
    fn partial_cmp(&self, other: &Box<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &Box<T>) -> bool { PartialOrd::lt(&**self, &**other) }
    #[inline]
    fn le(&self, other: &Box<T>) -> bool { PartialOrd::le(&**self, &**other) }
    #[inline]
    fn ge(&self, other: &Box<T>) -> bool { PartialOrd::ge(&**self, &**other) }
    #[inline]
    fn gt(&self, other: &Box<T>) -> bool { PartialOrd::gt(&**self, &**other) }
}

impl<T: Ord> Ord for Box<T> {
    #[inline]
    fn cmp(&self, other: &Box<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Eq> Eq for Box<T> {}

impl<T: Hash> Hash for Box<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: fmt::Display> fmt::Display for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Box({})", &**self)
    }
}

impl<T: fmt::Debug> fmt::Debug for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<I: Iterator> Iterator for Box<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> { (**self).next() }
    fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for Box<I> {
    fn next_back(&mut self) -> Option<I::Item> { (**self).next_back() }
}

impl<I: ExactSizeIterator> ExactSizeIterator for Box<I> {}

