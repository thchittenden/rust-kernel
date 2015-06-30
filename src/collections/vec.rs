//!
//! This module contains a growable vector.
//!
use core::prelude::*;
use core::ops::{Index, IndexMut};
use core::{ptr, mem, marker};
use core::slice;
use core::cmp::max;
use core::intrinsics::{drop_in_place, copy_nonoverlapping};
use alloc::{allocate_raw, deallocate_raw, reallocate_raw};
use util::{KernResult, KernResultEx};

/// A growable vector.
pub struct Vec<T> {
    raw: *mut T,
    cap: usize,
    len: usize
}

impl<T> Vec<T> {
    
    /// Creates a new vector with the given capacity.
    pub fn new(capacity: usize) -> KernResult<Vec<T>> {
        let size = capacity * mem::size_of::<T>();
        let align = mem::min_align_of::<T>();
        let addr = try!(allocate_raw(size, align));
        Ok(Vec {
            raw: addr as *mut T,
            cap: capacity,
            len: 0
        })
    }

    /// Clones a vector.
    pub fn clone(&self) -> KernResult<Vec<T>> where T: Clone {
        let size = self.len * mem::size_of::<T>();
        let align = mem::min_align_of::<T>();
        let addr = try!(allocate_raw(size, align));
        let vec = Vec {
            raw: addr as *mut T,
            cap: self.len,
            len: self.len,
        };
        for i in 0..self.len {
            // Perform a ptr::write so we don't try to drop the contents of the destination.
            unsafe { ptr::write(vec.raw.offset(i as isize), self[i].clone()) };
        }
        Ok(vec)
    }

    /// Returns the current number of elements in the vector.
    pub fn len(&self) -> usize {
        self.len
    }

    fn resize(&mut self, new_cap: usize) -> KernResult<()> {
        assert!(self.len < new_cap);
        let old_addr = self.raw as usize;
        let old_cap = self.cap;
        let old_size = old_cap * mem::size_of::<T>();
        let new_size = new_cap * mem::size_of::<T>();
        let align = mem::min_align_of::<T>();
        let new_addr = try!(reallocate_raw(old_addr, old_size, new_size, align));
        self.raw = new_addr as *mut T;
        self.cap = new_cap;
        Ok(())
    }

    pub fn reserve(&mut self, count: usize) -> KernResult<()> {
        assert!(self.cap >= self.len);
        let gap = self.cap - self.len;
        if gap < count {
            let new_cap = self.cap + count - gap;
            self.resize(new_cap)
        } else {
            Ok(())
        }
    }

    /// Sets the length to the given value. 
    ///
    /// # Safety
    ///
    /// This is unsafe because it will not drop any values that were discarded due to shrinking the
    /// vector and may allow access to uninitialized memory due to growing the vector.
    ///
    /// # Panics
    ///
    /// This function panics if we attempt to set the length to a value greater than the capacity.
    pub unsafe fn set_len(&mut self, len: usize) {
        assert!(len <= self.cap);
        self.len = len;
    }

    /// Attempts to push an element onto the queue. If the vector cannot allocate enough space to
    /// grow it returns Err(val), otherwise it returns Ok(())
    pub fn push(&mut self, val: T) -> KernResultEx<(), T> {
        if self.len < self.cap {
            // Have enough space, just perform the write.
            unsafe { ptr::write(self.raw.offset(self.len as isize), val) };
            self.len += 1;
            Ok(())
        } else {
            // Need to reallocate!
            let new_cap = max(self.cap, 1) * 2;
            try!(self.resize(new_cap), val);
            unsafe { ptr::write(self.raw.offset(self.len as isize), val) };
            self.len += 1;
            Ok(())
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

    pub fn split_at(&mut self, idx: usize) -> KernResult<Vec<T>> {
        if idx >= self.len {
            Vec::new(0)
        } else {
            let new_count = self.len - idx;
            let mut new = try!(Vec::new(new_count));
            new.len = new_count;
            unsafe { 
                copy_nonoverlapping(self.raw.offset(idx as isize), new.raw, new_count);
            }
            self.len = idx;
            Ok(new)
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // We know this is safe because we know we've allocated at least self.len entries.
        unsafe { slice::from_raw_parts(self.raw, self.len) }
    }

    pub unsafe fn as_slice_full(&self) -> &[T] {
        slice::from_raw_parts(self.raw, self.cap)
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // We know this is safe because we know we've allocated at least self.len entries.
        unsafe { slice::from_raw_parts_mut(self.raw, self.len) }
    }

    pub unsafe fn as_mut_slice_full(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.raw, self.cap)
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.len != mem::POST_DROP_USIZE {
            for i in 0 .. self.len {
                unsafe { drop_in_place(self.raw.offset(i as isize)) };
            }
            let size = self.cap * mem::size_of::<T>();
            deallocate_raw(self.raw as usize, size);
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(mut self) -> IntoIter<T> {
        // Prevent the vector from being dropped. We will deallocate the memory when we drop the
        // IntoIter.
        let len = self.len;
        self.len = mem::POST_DROP_USIZE;
        IntoIter {
            raw: self.raw,
            idx: 0,
            len: len,
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


