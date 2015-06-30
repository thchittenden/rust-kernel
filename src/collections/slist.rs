use alloc::boxed::Box;
use core::prelude::*;
use core::marker;
use link::HasSingleLink;
use raw::Raw;

pub struct SList<T: HasSingleLink<T> + ?Sized> {
    len: usize, 
    top: Option<Box<T>>,
}

impl<T: HasSingleLink<T> + ?Sized> SList<T> {
       
    /// Creates a new SList.
    pub fn new() -> SList<T> {
        SList::default() 
    }

    /// Pushes an element to the front of the list.
    pub fn push(&mut self, mut new_head: Box<T>) {
        assert!(new_head.slink().link.is_none());
        new_head.slink_mut().link = self.top.take();
        self.top = Some(new_head);
        self.len += 1;
    }

    /// Tries to pop an element from the list. Returns None if there are no elements in the list,
    /// Some(elem) otherwise.
    pub fn pop(&mut self) -> Option<Box<T>> {
        self.top.take().map(|mut top| {
            self.top = top.slink_mut().link.take();
            self.len -= 1;
            top
        })
    }

    /// Returns the number of elements in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Remove the first node in the list that satisfies the given condition.
    //  This was like the hardest function in Rust I've ever written. Why.
    pub fn remove_where<F: Fn(&T) -> bool>(&mut self, cond: F) -> Option<Box<T>> {
        self.top.take().and_then(|mut top| {
            if cond(&*top) {
                // The top element matched. Remove it and set the top to the next pointer.
                self.top = top.slink_mut().link.take();
                self.len -= 1;
                Some(top)
            } else {
                // The top element didn't match. Reset the top pointer and try to find the element
                // to be removed.
                self.top = Some(top);
                let mut res: Option<Box<T>> = None;
                for item in self.iter_mut() {
                    item.slink_mut().link = item.slink_mut().link.take().and_then(|mut next| {
                        if cond(&*next) {
                            // Found the node satisfying the condition. Remove it and return it.
                            let nextnext = next.slink_mut().link.take();
                            res = Some(next);
                            nextnext
                        } else {
                            Some(next)
                        }
                    })
                }
                if res.is_some() {
                    self.len -= 1;
                }
                res
            }
        })
    }

    /// Borrows the first element in the list that fulfills a given condition.
    pub fn borrow_where<F: Fn(&T) -> bool>(&self, cond: F) -> Option<&T> {
        for item in self {
            if cond(item) {
                return Some(item)
            }
        }
        None
    }

    pub fn borrow_mut_where<F: Fn(&T) -> bool> (&mut self, cond: F) -> Option<&mut T> {
        for item in self {
            if cond(item) {
                return Some(item)
            }
        }
        None
    }

    pub fn iter(&self) -> Iter<T> {
        Iter { 
            top: self.top.as_ref().map(|top| &**top)
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            top: self.top.as_mut().map(|top| unsafe { Raw::from_box(&mut *top) }),
            _marker: marker::PhantomData
        }
    }
}

impl<'a, T: HasSingleLink<T> + ?Sized> IntoIterator for &'a SList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T: HasSingleLink<T> + ?Sized> IntoIterator for &'a mut SList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T: HasSingleLink<T> + ?Sized> Default for SList<T> {
    fn default() -> SList<T> {
        SList {
            len: 0,
            top: None,
        }
    }   
}

pub struct Iter<'a, T: 'a + ?Sized> {
    top: Option<&'a T>,
}

pub struct IterMut<'a, T: 'a + ?Sized> {
    top: Option<Raw<T>>,
    _marker: marker::PhantomData<&'a mut T>
}

impl<'a, T: HasSingleLink<T> + ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.top.take().map(|top| {
            self.top = top.slink().link.as_ref().map(|next| &**next);
            top
        })
    }
}

impl<'a, T: HasSingleLink<T> + ?Sized> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        self.top.take().map(|mut top| {
            self.top = top.slink_mut().link.as_mut().map(|next| unsafe { Raw::from_box(next) });
            unsafe { top.as_mut() }
        })
    }
}

