use core::prelude::*;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::mem;

pub struct Global<T> {
    pub elem: UnsafeCell<Option<T>>
}
#[macro_export]
macro_rules! global_init {
    () => ({ 
        use core::option::Option::None;
        use core::cell::UnsafeCell;
        Global { elem: UnsafeCell { value: None } } 
    });
}

unsafe impl<T> Sync for Global<T> { }

impl<T> Global<T> {

    pub fn init(&self, elem: T) {
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

