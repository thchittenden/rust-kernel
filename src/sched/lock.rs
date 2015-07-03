//! This module contains the scheduler lock. 
//!
//! Currently this is just implemented using disable_interrupts, however it is general enough so
//! that it may be implemented using spin locks if SMP is supported in the future.
//!
//! SchedLocks are a little dangerous at the moment as it allows the user to obtain multiple
//! mutable borrows of whatever the contents are by calling `lock()` twice. I'm not sure whether I
//! want to make the lock non-reentrant or just mark lock as unsafe and live with it.

use core::prelude::*;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use util::asm;

pub struct SchedLockGuard<'a, T: 'a> {
    reenable: bool,
    data: &'a UnsafeCell<T>,
}
pub struct SchedLock<T> {
    data: UnsafeCell<T>
}

impl<T> SchedLock<T> {
   
    pub const fn new(data: T) -> SchedLock<T> {
        SchedLock {
            data: UnsafeCell::new(data)
        }
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
