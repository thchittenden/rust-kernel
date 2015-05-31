//!
//! A pointer type for non-heap allocations. 
//!
//! Raw boxes are manually managed owned pointers. They are used for any allocation that is not
//! performed on the heap. This includes physical memory frames.  Because these boxes are not
//! managed it is possible they can be leaked. It is vital that all users of RawBoxes must be
//! careful not to leak them!
//!
use core::prelude::*;
use core::ptr::Unique;
use core::hash;
use core::hash::Hash;
use core::cmp::Ordering;
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::mem;

/// An owned pointer.
pub struct RawBox<T>(Unique<T>);

/// A marker trait indicating we can freely cast a box of this type to a box of another type given
/// it will fit. This is used for objects like Frame that represent 4K of unallocated memory.
pub trait Unallocated { 
    /// Returns the amount of free space the struct represents. mem::size_of does not suffice in
    /// this case as we don't always want to fill the entire area of memory with a struct (e.g. in
    /// the case of a Frame).
    fn get_free_size(&self) -> usize;
}

impl<T> RawBox<T> {
   
    /// Constructs an owned pointer from a unique memory address.
    pub fn from_uniq(addr: Unique<T>) -> RawBox<T> {
        RawBox(addr) 
    }

    /// Constructs an owned pointer from a memory address.
    /// 
    /// # Safety
    ///
    /// This is unsafe because the caller must ensure that the address is in fact unique.
    pub unsafe fn from_raw(addr: *mut T) -> RawBox<T> {
        RawBox(Unique::new(addr))
    }

    /// Converts an owned pointer into a unique pointer.
    pub fn into_uniq(self) -> Unique<T> {
        self.0
    }

    /// Converts an owned pointer into a memory address.
    pub fn into_raw(mut self) -> *mut T {
        unsafe { self.0.get_mut() as *mut T }
    }

    /// Borrows the contents of the box.
    pub fn borrow(&self) -> &T {
        unsafe { self.0.get() }
    }

    /// Mutably borrows the contents of the box.
    pub fn borrow_mut(&mut self) -> &mut T {
        unsafe { self.0.get_mut() }
    }

}

impl <T: Unallocated> RawBox<T> {
   
    /// Convert a pointer to unallocated memory to some allocated memory.
    pub fn allocate<U>(mut self) -> RawBox<U> {
        assert!(mem::size_of::<U>() <= self.get_free_size());
        unsafe { RawBox::from_raw(self.into_raw() as *mut U) }
    }

}

impl<T> Deref for RawBox<T> {
    type Target = T;
    fn deref(&self) -> &T { 
        // We cannot implement this as &**self because it causes an infinite 
        // loop (trying to call deref!). I don't really know why this is 
        // because this is how deref is implemented for the standard library's
        // Box...
        unsafe { self.0.get() } 
    }
}

impl<T> DerefMut for RawBox<T> {
    fn deref_mut(&mut self) -> &mut T { 
        // See Deref.
        unsafe { self.0.get_mut() }
    }
}

impl<T: PartialEq> PartialEq for RawBox<T> {
    #[inline]
    fn eq(&self, other: &RawBox<T>) -> bool { PartialEq::eq(&**self, &**other) }
    #[inline]
    fn ne(&self, other: &RawBox<T>) -> bool { PartialEq::ne(&**self, &**other) }
}

impl<T: PartialOrd> PartialOrd for RawBox<T> {
    #[inline]
    fn partial_cmp(&self, other: &RawBox<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &RawBox<T>) -> bool { PartialOrd::lt(&**self, &**other) }
    #[inline]
    fn le(&self, other: &RawBox<T>) -> bool { PartialOrd::le(&**self, &**other) }
    #[inline]
    fn ge(&self, other: &RawBox<T>) -> bool { PartialOrd::ge(&**self, &**other) }
    #[inline]
    fn gt(&self, other: &RawBox<T>) -> bool { PartialOrd::gt(&**self, &**other) }
}

impl<T: Ord> Ord for RawBox<T> {
    #[inline]
    fn cmp(&self, other: &RawBox<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Eq> Eq for RawBox<T> {}

impl<T: Hash> Hash for RawBox<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: fmt::Display> fmt::Display for RawBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for RawBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<I: Iterator> Iterator for RawBox<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> { (**self).next() }
    fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for RawBox<I> {
    fn next_back(&mut self) -> Option<I::Item> { (**self).next_back() }
}

impl<I: ExactSizeIterator> ExactSizeIterator for RawBox<I> {}


