use core::prelude::*;
use core::atomic::{AtomicUsize, Ordering};
use core::ops::{Deref, CoerceUnsized};
use core::marker::Unsize;
use core::fmt;
use boxed::Box;

/// Indicates that a type has an internal reference count.
pub trait HasRc {
    /// Returns the reference count.
    fn get_count(&self) -> &AtomicUsize;
}

/// A reference counted pointer.
pub struct Rc<T: ?Sized + HasRc> {
    value: *mut T
}


/// Allow casting from a Box<T> to a Box<U> where T implements U.
impl<T: ?Sized+Unsize<U>+HasRc, U: ?Sized+HasRc> CoerceUnsized<Rc<U>> for Rc<T> {}

impl<T: ?Sized + HasRc> Rc<T> {
    /// Constructs a new RC type.
    pub fn new(val: Box<T>) -> Rc<T> {
        val.get_count().store(1, Ordering::Relaxed);
        Rc { value: unsafe { val.into_raw() } }
    }
}

impl<T: ?Sized + HasRc> Clone for Rc<T> {
    fn clone(&self) -> Rc<T> {
        unsafe { &*self.value }.get_count().fetch_add(1, Ordering::Relaxed);
        Rc { value: self.value }
    }
}

impl<T: ?Sized + HasRc> Drop for Rc<T> {
    fn drop(&mut self) {
        let count = unsafe { &*self.value }.get_count().fetch_sub(1, Ordering::Relaxed);
        if count == 1 {
            // We were the last reference. Drop the contents.
            unsafe { drop(Box::from_raw(self.value)) };
        }
    }
}

impl <T: ?Sized + HasRc> Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.value }
    }

}

impl <T: ?Sized + HasRc + fmt::Debug> fmt::Debug for Rc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rc({:?})", self.deref()) 
    }
}

