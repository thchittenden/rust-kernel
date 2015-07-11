use core::prelude::*;
use core::atomic::{AtomicUsize, Ordering};
use core::ops::{Deref, CoerceUnsized};
use core::marker::Unsize;
use core::any::Any;
use core::fmt;
use boxed::Box;
logger_init!(Debug);

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

    /// Constructs a ref-counted pointer from a reference.
    ///
    /// # Panics
    ///
    /// This function panics if the value does not have at least once reference to it already. This
    /// would indicate that the value may not have originated from a Box and thus it may be unsafe
    /// to try to deallocate.
    pub fn from_ref(val: &T) -> Rc<T> {
        assert!(val.get_count().fetch_add(1, Ordering::Relaxed) > 0);
        Rc { value: val as *const T as *mut T } 
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
        trace!("dropping rc 0x{:x}", self.value as *const () as usize);
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

/// A reference counted Any type. This is necessary in order to store an Any type in an Rc pointer.
/// It is necessary to have the `as_any` method because we can't implement methods like
/// `downcast_ref` that are defined in the `impl Any` from an `RcAny` trait object.
pub trait RcAny : HasRc + Any { 
    fn as_any(&self) -> &Any;
}
impl <T: Any + HasRc> RcAny for T {
    fn as_any(&self) -> &Any {
        self
    }
}


/// A reference counted pointer.
pub struct RcPtr<T: ?Sized> {
    rc: AtomicUsize,
    val: Box<T>,
}

impl<T: ?Sized> RcPtr<T> {
    pub fn new(val: Box<T>) -> RcPtr<T> {
        RcPtr {
            rc: AtomicUsize::new(1),
            val: val
        }
    }
}

impl<T: ?Sized> HasRc for RcPtr<T> {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}

impl<T: ?Sized> Deref for RcPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.val.deref()
    }
}
