use core::prelude::*;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// A wrapper type for global variables that dynamically enforces that they are initialized before
/// their first use and that they are only initialized once.
pub struct Global<T> {
    /// The internal element type.
    elem: UnsafeCell<Option<T>>
}

unsafe impl<T> Sync for Global<T> { }

impl<T> Global<T> {

    pub const fn new() -> Global<T> {
        Global { elem: UnsafeCell::new(None) }
    }

    /// Initializes a global value.
    ///
    /// # Panics
    ///
    /// This function panics if the global has already been initialized.
    pub fn init(&self, elem: T) {
        assert!(unsafe { (*self.elem.get()).is_none() });
        unsafe { *self.elem.get() = Some(elem); }
    }

}

impl<T> Deref for Global<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { (*self.elem.get()).as_ref().expect("attempted to use an uninitialized global") }
    }
}

impl<T> DerefMut for Global<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { (*self.elem.get()).as_mut().expect("attempted to use an uninitialized global") }
    }
}

