use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::prelude::*;
use condvar::CondVar;
use mutex::Mutex;

pub struct SemaphoreGuard<'a, T: 'a> {
    lock: &'a Semaphore<T>,
    data: &'a UnsafeCell<T>
}

// Fields are marked public so they may be statically initialized.
pub struct SemaphoreInternal {
    count: usize
}

pub struct Semaphore<T> {
    semint: Mutex<SemaphoreInternal>,
    data: UnsafeCell<T>,
    cv: CondVar
}

impl<T> Semaphore<T> {
    
    pub fn new(count: usize, data: T) -> Semaphore<T> {
        Semaphore {
            semint: Mutex::new(SemaphoreInternal { count: count }),
            data: UnsafeCell::new(data),
            cv: CondVar::new(),
        }
    }

    pub fn acquire(&self) -> Option<SemaphoreGuard<T>> {
        let mut data = self.semint.lock();
        while data.count <= 0 {
            data = self.cv.wait(data);
        }
        data.count -= 1;

        Some (SemaphoreGuard {
            lock: &*self,
            data: &self.data
        })
    }

    fn release(&self) {
        self.cv.signal();
    }

}

unsafe impl <T> Sync for Semaphore<T> { }

impl <'mutex, T> Deref for SemaphoreGuard<'mutex, T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        return unsafe { &*self.data.get() };
    }
}

impl <'mutex, T> DerefMut for SemaphoreGuard<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        return unsafe { &mut *self.data.get() };
    }
}

impl <'mutex, T> Drop for SemaphoreGuard<'mutex, T> {
    fn drop(&mut self) {
        self.lock.release();
    }
}
