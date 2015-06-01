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
    pub count: usize
}

pub struct Semaphore<T> {
    pub semint: Mutex<SemaphoreInternal>,
    pub data: UnsafeCell<T>,
    pub cv: CondVar
}

#[macro_export]
macro_rules! static_semaphore {
    ($count:expr, $data:expr) => ({
        use core::cell::UnsafeCell;
        Semaphore {
            semint: static_mutex!(SemaphoreInternal {
                count: $count,
            }),
            data: UnsafeCell { value: $data },
            cv: static_condvar!()
        }
    });
}

impl<T> Semaphore<T> {
    
    pub fn new(count: usize, data: T) -> Semaphore<T> {
        static_semaphore!(count, data)
    }

    pub fn acquire(&self) -> Option<SemaphoreGuard<T>> {
        let mut data = self.semint.lock();
        while data.count <= 0 {
            self.cv.wait(&data);
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
