use core::prelude::*;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use mutex::Mutex;
use condvar::CondVar;

/// An RAII style lock for readers.
pub struct ReaderGuard<'a, T: 'a> {
    dropped: bool,
    lock: &'a RWLock<T>,
    data: &'a UnsafeCell<T>,
}

/// An RAII style lock for derived contents of a reader lock. This is useful in cases such as the
/// virtual file system where we want to create an iterator that is scoped to the lock. 
pub struct ReaderGuardMap<'a, T: 'a, U: 'a> {
    lock: &'a RWLock<T>,
    data: U
}

pub struct WriterGuard<'a, T: 'a> {
    lock: &'a RWLock<T>,
    data: &'a UnsafeCell<T>,
}

pub struct RWLockState {
    nreaders: usize,
    nwriters: usize,
    nreaders_waiting: usize,
    nwriters_waiting: usize,
}

impl RWLockState {
    fn new() -> RWLockState {
        RWLockState { 
            nreaders: 0,
            nwriters: 0,
            nreaders_waiting: 0,
            nwriters_waiting: 0,
        }
    }
}

pub struct RWLock<T> {
    state: Mutex<RWLockState>,
    writer_cond: CondVar,
    reader_cond: CondVar,
    data: UnsafeCell<T>,
}

impl<T> RWLock<T> {

    pub fn new(data: T) -> RWLock<T> {
        RWLock {
            state: Mutex::new(RWLockState::new()),
            writer_cond: CondVar::new(),
            reader_cond: CondVar::new(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock_reader(&self) -> ReaderGuard<T> {
        let mut state = self.state.lock();
        state.nreaders_waiting += 1;
        while state.nwriters > 0 {
            state = self.reader_cond.wait(state)
        }
        state.nreaders_waiting -= 1;
        state.nreaders += 1;
        ReaderGuard {
            dropped: false,
            lock: &self,
            data: &self.data
        }
    }

    pub fn lock_writer(&self) -> WriterGuard<T> {
        let mut state = self.state.lock();
        state.nwriters_waiting += 1;
        while state.nreaders > 0 && state.nwriters > 0 {
            state = self.writer_cond.wait(state)
        }
        state.nwriters_waiting -= 1;
        state.nwriters += 1;
        WriterGuard {
            lock: &self,
            data: &self.data,
        }
    }

    fn unlock_reader(&self) {
        let mut state = self.state.lock();
        assert!(state.nreaders > 0);
        assert!(state.nwriters == 0);
        state.nreaders -= 1;
        if state.nreaders == 0 {
            if state.nwriters_waiting > 0 {
                self.writer_cond.signal()
            }
        }
    }

    fn unlock_writer(&self) {
        let mut state = self.state.lock();
        assert!(state.nwriters == 1);
        assert!(state.nreaders == 0);
        state.nwriters -= 1;
        if state.nreaders_waiting > 0 {
            self.reader_cond.broadcast()
        } else if state.nwriters_waiting > 0 {
            self.writer_cond.signal()
        }
    }

}

unsafe impl <T> Sync for RWLock<T> { }

impl<'lock, T> ReaderGuard<'lock, T> {
    pub fn map<U, F: Fn(&'lock T) -> U>(mut self, f: F) -> ReaderGuardMap<'lock, T, U> {
        self.dropped = true;
        let u: U = f(unsafe { &*self.data.get() });
        ReaderGuardMap {
            lock: self.lock,
            data: u,
        }
    }
}

impl<'lock, T> Deref for ReaderGuard<'lock, T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'lock, T, U: 'lock> Deref for ReaderGuardMap<'lock, T, U> {
    type Target = U;
    fn deref<'a>(&'a self) -> &'a U {
        unsafe { &self.data }
    }
}

impl<'lock, T> Deref for WriterGuard<'lock, T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'lock, T> DerefMut for WriterGuard<'lock, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

// It's safe to have a DerefMut trait for a ReaderGuardMap because the ReaderGuardMap owns the
// data.
impl<'lock, T, U: 'lock> DerefMut for ReaderGuardMap<'lock, T, U> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut U {
        unsafe { &mut self.data }
    }
}

impl<'lock, T> Drop for ReaderGuard<'lock, T> {
    fn drop(&mut self) {
        if !self.dropped {
            self.dropped = true;
            self.lock.unlock_reader();
        }
    }
}

impl<'lock, T, U> Drop for ReaderGuardMap<'lock, T, U> {
    fn drop(&mut self) {
        self.lock.unlock_reader();
    }
}

impl<'lock, T> Drop for WriterGuard<'lock, T> {
    fn drop(&mut self) {
        self.lock.unlock_writer();
    }
}
