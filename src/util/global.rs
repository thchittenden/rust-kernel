use core::prelude::*;
use core::ops::Deref;
use core::mem;

pub struct Global<T> {
    pub elem: Option<T>
}
#[macro_export]
macro_rules! global_init {
    () => ({ 
        use core::cell::UnsafeCell;
        Global { elem: None } 
    });
}

impl<T> Global<T> {
    
    pub fn init(&self, elem: T) {
        unsafe { 
            // Subverting the mutability of the global for initial assignment.
            let ptr: *mut T = mem::transmute(&self.elem);
            *ptr = elem;
        }
    }

}

impl<T> Deref for Global<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.elem.as_ref().expect("attempted to use an uninitialized global")
    }
}
