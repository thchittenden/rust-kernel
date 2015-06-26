#![crate_name="mutex"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude)]
#![no_std]
//!
//! This module contains the kernel's mutex implementation.
//!
//! This was made as a separate module in order to break many circular dependencies. Many low level
//! crates (alloc, console, io) require some form of mutual exclusion, however higher level crates
//! like sync depend on collections which depends on alloc thus we can't put mutex in sync. 
//!
//! Since the mutex needs to interact with the scheduler and the scheduler relies on crates that
//! rely on the mutex, we use an `extern fn` to break the last cycle.
//!
//! The mutex is implemented using the bakery algorithm and a yield-to-owner loop. TODO it
//! currently uses a yield-to-anyone loop.
//!

extern crate core;

use core::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::prelude::*;

// This is our entry point to the scheduler. This prevents the need for libmutex to rely on
// libsched which indirectly needs to rely on libmutex.
#[allow(improper_ctypes)] // This doesn't go to C!
extern {
    fn sched_yield(tid: Option<usize>);
}

/// An RAII-style object used to unlock the mutex.
pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
    data: &'a UnsafeCell<T>,
}

/// The mutex object. Fields are marked public to enable static initialization.
pub struct Mutex<T> {

    /// The current ticket. Whoever holds this ticket is allowed to access the underlying data.
    pub curr_ticket: AtomicUsize,

    /// The next ticket to be handed out to callers.
    pub next_ticket: AtomicUsize,

    /// The underlying data controlled by the mutex.
    pub data: UnsafeCell<T>,
}

/// Statically initializes a mutex.
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
   
    /// Constructs a new mutex for the given data.
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            curr_ticket: AtomicUsize::new(0),
            next_ticket: AtomicUsize::new(0),
            data: UnsafeCell::new(data)
        }
    }

    /// Returns an RAII style lock on the contents of the mutex. This function blocks until we own
    /// the mutex.
    pub fn lock(&self) -> MutexGuard<T> {
        // Take a ticket.
        let my_ticket = self.next_ticket.fetch_add(1, Ordering::SeqCst);

        // Wait for our ticket to come up.
        while my_ticket != self.curr_ticket.load(Ordering::SeqCst) {
            // TODO don't yield to anyone, yield to someone! (but not Some(1))
            // We know this is safe because the scheduler implements sched_yield.
            unsafe { sched_yield(None) };
        }

        // We now have the lock.
        MutexGuard {
            lock: &self,
            data: &self.data
        }
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
        unsafe { &*self.data.get() }
    }
}

impl <'mutex, T> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl <'mutex, T> Drop for MutexGuard<'mutex, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
