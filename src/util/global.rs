use core::prelude::*;
use core::ops::Deref;
use core::mem;

pub struct Global<T> {
    pub elem: Option<T>
}
#[macro_export]
macro_rules! global_init {
    () => ({ 
        use core::option::Option::None;
        Global { elem: None } 
    });
}

impl<T> Global<T> {
    
    pub fn init(&self, elem: T) {
        unsafe { 
            // Subverting the mutability of the global for initial assignment.
            let ptr: *mut Option<T> = mem::transmute(&self.elem);
            *ptr = Some(elem);
        }
    }

}

impl<T> Deref for Global<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.elem.as_ref().expect("attempted to use an uninitialized global")
    }
}
