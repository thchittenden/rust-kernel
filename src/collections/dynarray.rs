//! This module contains the definition of a dynamically resizeable array. 
//!
//! The contents of the array must implement Default so that when new entries are added they can be
//! initialized properly.
//!
use core::prelude::*;
use core::{mem, ptr, marker};
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

    pub fn clone(&self) -> Option<DynArray<T>> where T: Clone {
        let size = self.len * mem::size_of::<T>();
        let align = mem::min_align_of::<T>();
        let addr = try_op!(allocate_raw(size, align));
        let mut dyn = DynArray {
            raw: addr as *mut T,
            len: self.len,
        };
        for i in 0..self.len {
            dyn[i] = self[i].clone();
        }
        Some(dyn)
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

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            Some(unsafe { &*self.raw.offset(idx as isize) })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len {
            Some(unsafe { &mut*self.raw.offset(idx as isize) })
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            raw: self.raw,
            idx: 0,
            len: self.len,
            _marker: marker::PhantomData
        }
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

impl<T: Default> Index<usize> for DynArray<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        self.get(idx).unwrap()
    }
}

impl<T: Default> IndexMut<usize> for DynArray<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        self.get_mut(idx).unwrap()
    }
}

impl<T> IntoIterator for DynArray<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(mut self) -> IntoIter<T> {
        let len = self.len;
        self.len = mem::POST_DROP_USIZE;
        IntoIter {
            raw: self.raw,
            idx: 0,
            len: len,
        }
    }
}   

impl<'a, T: Default> IntoIterator for &'a DynArray<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
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

