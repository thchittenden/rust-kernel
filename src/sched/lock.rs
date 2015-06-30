//! This module contains the scheduler lock. 
//!
//! Currently this is just implemented using disable_interrupts, however it is general enough so
//! that it may be implemented using spin locks if SMP is supported in the future.
//!
//! In order to prevent the user from acquiring two mutable references to the content, the lock
//! tracks whether anyone is currently holding the lock when `lock` is called. Since iterrupts are
//! disabled when this lock is taken, the only way this could happen is if the user called `lock`
//! twice. If the user attempts to use this the lock will panic.
//! 
use core::prelude::*;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::atomic::{AtomicBool, Ordering};
use util::asm;

pub struct SchedLockGuard<'a, T: 'a> {
    lock: &'a SchedLock<T>,
    data: &'a UnsafeCell<T>,
}
pub struct SchedLock<T> {
    borrowed: AtomicBool,
    reenable: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SchedLock<T> {
   
    pub const fn new(data: T) -> SchedLock<T> {
        SchedLock {
            borrowed: AtomicBool::new(false),
            reenable: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SchedLockGuard<T> {
        let reenable = asm::interrupts_enabled();
        if reenable {
            asm::disable_interrupts();
        }
        assert!(self.borrowed.load(Ordering::Relaxed) == false);
        self.borrowed.store(true, Ordering::Relaxed);
        self.reenable.store(reenable, Ordering::Relaxed);
        
        SchedLockGuard {
            lock: self,
            data: &self.data
        }
    }

    fn unlock(&self) {
        self.borrowed.store(false, Ordering::Relaxed);
        if self.reenable.load(Ordering::Relaxed) {
            asm::enable_interrupts();
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
        self.lock.unlock()
    }
}
