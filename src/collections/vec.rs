//!
//! This module contains a growable vector.
//!
use core::prelude::*;
use core::ops::{Index, IndexMut};
use core::{ptr, mem, marker};
use alloc::{allocate_raw, deallocate_raw, reallocate_raw};

/// A growable vector.
pub struct Vec<T> {
    raw: *mut T,
    cap: usize,
    len: usize
}

impl<T> Vec<T> {
    
    /// Creates a new vector with the given capacity.
    pub fn new(capacity: usize) -> Option<Vec<T>> {
        let size = capacity * mem::size_of::<T>();
        let align = mem::min_align_of::<T>();
        let alloc = allocate_raw(size, align);
        alloc.map(|addr| {
            Vec {
                raw: addr as *mut T,
                cap: capacity,
                len: 0
            }
        })
    }

    /// Returns the current number of elements in the vector.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Attempts to push an element onto the queue. If the vector cannot allocate enough space to
    /// grow it returns Err(val), otherwise it returns Ok(())
    #[must_use]
    pub fn push(&mut self, val: T) -> Result<(), T> {
        if self.len < self.cap {
            // Have enough space, just perform the write.
            unsafe { ptr::write(self.raw.offset(self.len as isize), val) };
            self.len += 1;
            Ok(())
        } else {
            // Need to reallocate!
            let old_addr = self.raw as usize;
            let old_size = self.cap * mem::size_of::<T>();
            let new_size = old_size * 2; // TODO 2?
            let align = mem::min_align_of::<T>();
            match reallocate_raw(old_addr, old_size, new_size, align) {
                Ok(new_addr) => {
                    self.raw = new_addr as *mut T;
                    self.cap *= 2;
                    unsafe { ptr::write(self.raw.offset(self.len as isize), val) };
                    self.len += 1;
                    Ok(())
                }
                Err(_) => Err(val)
            }
        }
    }

    /// Attempts to pop an element from the vector. If the vector is empty, returns None, otherwise
    /// returns Some(elem).
    pub fn pop(&mut self) -> Option<T>  {
        if self.len > 0 {
            let res = unsafe { ptr::read(self.raw.offset(self.len as isize)) };
            self.len -= 1;
            Some(res)
        } else {
            None
        }
    }

}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.len != mem::POST_DROP_USIZE {
            let size = self.cap * mem::size_of::<T>();
            deallocate_raw(self.raw as usize, size);
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            raw: self.raw,
            idx: 0,
            len: self.len,
        }
    }
}   

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        Iter {
            raw: self.raw,
            idx: 0,
            len: self.len,
            _marker: marker::PhantomData
        }
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        assert!(idx < self.len);
        unsafe { &*self.raw.offset(idx as isize) }
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        assert!(idx < self.len);
        unsafe { &mut*self.raw.offset(idx as isize) }
    }
}

/// A consuming iterator.
pub struct IntoIter<T> {
    raw: *mut T,
    idx: usize,
    len: usize,
}

/// A borrowing iterator.
pub struct Iter<'a, T: 'a> {
    raw: *const T,
    idx: usize,
    len: usize,
    _marker: marker::PhantomData<&'a T>
}

/// A mutably borrowing iterator.
pub struct IterMut<'a, T: 'a> {
    raw: *mut T,
    idx: usize,
    len: usize,
    _marker: marker::PhantomData<&'a mut T>
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.idx == self.len {
            None
        } else {
            let res = Some(unsafe { ptr::read(self.raw.offset(self.idx as isize)) });
            self.idx += 1;
            res
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // If we haven't already been dropped, deallocate the space.
        if self.idx != mem::POST_DROP_USIZE {
            let addr = self.raw as usize;
            let size = self.len * mem::size_of::<T>();
            deallocate_raw(addr, size);
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.idx == self.len {
            None 
        } else {
            let res = Some(unsafe { &*self.raw.offset(self.idx as isize) });
            self.idx += 1;
            res
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        if self.idx == self.len {
            None
        } else {
            let res = Some(unsafe { &mut *self.raw.offset(self.idx as isize) });
            self.idx += 1;
            res
        }
    }
}


