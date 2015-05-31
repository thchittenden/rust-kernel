//! This module contains the scheduler lock. 
//!
//! Currently this is just implemented using disable_interrupts, however it is general enough so
//! that it may be implemented using spin locks if SMP is supported in the future.

use core::prelude::*;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use util::asm;

pub struct SchedLockGuard<'a, T: 'a> {
    reenable: bool,
    data: &'a UnsafeCell<T>,
}
pub struct SchedLock<T> {
    pub data: UnsafeCell<T>
}

#[macro_export]
macro_rules! static_schedlock {
    ($data:expr) => ({
        use $crate::lock::SchedLock;
        use core::cell::UnsafeCell;
        SchedLock { 
            data: UnsafeCell { value: $data }  
        }
    });
}

impl<T> SchedLock<T> {
    
    pub fn new(data: T) -> SchedLock<T> {
        static_schedlock!(data)
    }

    pub fn lock<'a>(&'a self) -> SchedLockGuard<'a, T> {
        let reenable = asm::interrupts_enabled();
        if reenable {
            asm::disable_interrupts();
        }
        
        SchedLockGuard {
            reenable: reenable,
            data: &self.data
        }
    }

}

unsafe impl<T> Sync for SchedLock<T> { }

impl <'lock, T> Deref for SchedLockGuard<'lock, T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl <'lock, T> DerefMut for SchedLockGuard<'lock, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl <'lock, T> Drop for SchedLockGuard<'lock, T> {
    fn drop (&mut self) {
        if self.reenable {
            asm::enable_interrupts();
        }
    }
}
