#![crate_name="mutex"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

extern crate core;

use core::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::prelude::*;

// This is our entry point to the scheduler. This prevents the need for libmutex to rely on
// libsched which needs to rely on libmutex.
extern {
    fn sched_yield(tid: Option<usize>);
}

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
    data: &'a UnsafeCell<T>,
}

// Fields are marked public so they may be statically initialized.
pub struct Mutex<T> {
    pub curr_ticket: AtomicUsize,
    pub next_ticket: AtomicUsize,
    pub data: UnsafeCell<T>,
}

#[macro_export]
macro_rules! static_mutex {
    ($data:expr) => ({
        use core::atomic::ATOMIC_USIZE_INIT;
        use core::cell::UnsafeCell;
        use $crate::Mutex;
        Mutex {
            curr_ticket: ATOMIC_USIZE_INIT,
            next_ticket: ATOMIC_USIZE_INIT,
            data: UnsafeCell {
                value: $data
            }
        }
    });
}

impl <T> Mutex<T> {
    
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            curr_ticket: AtomicUsize::new(0),
            next_ticket: AtomicUsize::new(0),
            data: UnsafeCell::new(data)
        }
    }

    pub fn lock(&self) -> Option<MutexGuard<T>> {
        // Take a ticket.
        let my_ticket = self.next_ticket.fetch_add(1, Ordering::SeqCst);

        // Wait for our ticket to come up.
        while my_ticket != self.curr_ticket.load(Ordering::SeqCst) {
            // TODO don't yield to anyone, yield to someone! (but not Some(1))
            unsafe { sched_yield(None) };
        }

        // We now have the lock.
        Some(MutexGuard {
            lock: &self,
            data: &self.data
        })
    }

    fn unlock(&self) {
        // Notify next thread that it's their turn.
        self.curr_ticket.fetch_add(1, Ordering::SeqCst);
    }

}

unsafe impl <T> Sync for Mutex<T> { }

impl <'mutex, T> Deref for MutexGuard<'mutex, T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        return unsafe { &*self.data.get() };
    }
}

impl <'mutex, T> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        return unsafe { &mut *self.data.get() };
    }
}

impl <'mutex, T> Drop for MutexGuard<'mutex, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
