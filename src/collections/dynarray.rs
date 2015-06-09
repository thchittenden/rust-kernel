//! This module contains the definition of a dynamically resizeable array. 
//!
//! The contents of the array must implement Default so that when new entries are added they can be
//! initialized properly.
//!
use core::prelude::*;
use core::{mem, ptr};
use core::ops::{Index, IndexMut};
use core::intrinsics::drop_in_place;
use alloc::{allocate_raw, reallocate_raw, deallocate_raw};

/// A dynamicly resizable array. 
pub struct DynArray<T> {
    raw: *mut T,
    len: usize
}

impl<T: Default> DynArray<T> {
   
    /// Creates a new Dynamic Array of the given size.
    pub fn new(count: usize) -> Option<DynArray<T>> {
        let size = count * mem::size_of::<T>();
        let align = mem::min_align_of::<T>();
        allocate_raw(size, align).map(|addr| {
            let dyn = DynArray {
                raw: addr as *mut T,
                len: count
            };
            for i in 0 .. count {
                unsafe { ptr::write(dyn.raw.offset(i as isize), T::default()) };
            }
            dyn
        })
    }

    /// Attempts to change the size of the array and returns whether it was successful or not.
    #[must_use]
    pub fn resize(&mut self, new_count: usize) -> bool {
        let old_addr = self.raw as usize;
        let old_count = self.len;
        let old_size = old_count * mem::size_of::<T>();
        let new_size = new_count * mem::size_of::<T>();
        let align = mem::min_align_of::<T>();
        match reallocate_raw(old_addr, old_size, new_size, align) {
            Ok(new_addr) => {
                self.raw = new_addr as *mut T;
                self.len = new_count;
                for i in old_count .. new_count {
                    unsafe { ptr::write(self.raw.offset(i as isize), T::default()) };
                }
                true
            }
            Err(_) => false
        }
    }

    /// Returns the length of the array.
    pub fn len(&self) -> usize {
        self.len
    }

}

impl<T> Drop for DynArray<T> {
    fn drop(&mut self) {
        if self.len != mem::POST_DROP_USIZE {
            // Drop all the contents of the array.
            for i in 0..self.len {
                // We know it is safe to drop these because they were initialized with a valid value.
                unsafe { drop_in_place(self.raw.offset(i as isize)) };
            }

            // Deallocate the array itself.
            let addr = self.raw as usize;
            let size = self.len * mem::size_of::<T>();
            deallocate_raw(addr, size)
        }
    }
}

impl<T> Index<usize> for DynArray<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        assert!(idx < self.len);
        unsafe { &*self.raw.offset(idx as isize) }
    }
}

impl<T> IndexMut<usize> for DynArray<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        assert!(idx < self.len);
        unsafe { &mut*self.raw.offset(idx as isize) }
    }
}
