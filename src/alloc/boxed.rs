// Kernel box implementation. This was heavily lifted from the std library's Box.
use core::prelude::*;
use core::ptr;
use core::ptr::Unique;
use core::mem;
use core::hash;
use core::hash::Hash;
use core::cmp::Ordering;
use core::fmt;
use core::ops::{Deref, DerefMut};

//#[unsafe_no_drop_flag]
pub struct Box<T>(Unique<T>);

impl<T> Box<T> {
    pub fn new (x: T) -> Option<Box<T>> {
        ::allocate(x).map(Box)
    }
    pub fn new_aligned(x: T, align: usize) -> Option<Box<T>> {
        ::allocate_aligned(x, align).map(Box)
    }
}

impl <T> Drop for Box<T> {
    fn drop(&mut self) {
        //trace!("dropping 0x{:x}", &mut **self as *mut T as usize);

        // Swap a null pointer into the box.
        let mut val = unsafe { Unique::new(ptr::null_mut()) };
        mem::swap(&mut self.0, &mut val);

        // If we get a non-null pointer back, then we are the first 
        // call to the destructor so we should deallocate the pointer.
        if !val.is_null() {
            ::deallocate(val);
        }
    }
}

impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T { 
        // We cannot implement this as &**self because it causes an infinite 
        // loop (trying to call deref!). I don't really know why this is 
        // because this is how deref is implemented for the standard library's
        // Box...
        unsafe { self.0.get() } 
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T { 
        // See Deref.
        unsafe { self.0.get_mut() }
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
        fmt::Display::fmt(&**self, f)
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

