use core::prelude::*;
use core::ops::{Index, IndexMut};
use core::{ptr, mem, marker};

pub struct IntoIter<T> {
    raw: *mut T,
    idx: usize,
    len: usize,
}

pub struct Iter<'a, T: 'a> {
    raw: *const T,
    idx: usize,
    len: usize,
    _marker: marker::PhantomData<&'a T>
}

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
        if self.idx != mem::POST_DROP_USIZE {
            unimplemented!();
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

pub struct Vec<T> {
    raw: *mut T,
    cap: usize,
    len: usize
}

impl<T> Vec<T> {
    
    pub fn new() -> Vec<T> {
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn append(&mut self, val: T) -> bool {
        unimplemented!()
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
