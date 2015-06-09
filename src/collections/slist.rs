use alloc::boxed::Box;
use core::prelude::*;
use core::ops::DerefMut;
use core::marker;
use link::HasSingleLink;
use raw::Raw;

pub struct Iter<'a, T: 'a> {
    top: Option<&'a T>,
}

pub struct IterMut<'a, T: 'a> {
    top: Option<Raw<T>>,
    _marker: marker::PhantomData<&'a mut T>
}

impl<'a, T: HasSingleLink<T=T>> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.top.take().map(|top| {
            self.top = top.slink().link.as_ref().map(|next| &**next);
            top
        })
    }
}

impl<'a, T: HasSingleLink<T=T>> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        self.top.take().map(|mut top| {
            self.top = top.slink_mut().link.as_mut().map(|next| unsafe { Raw::from_box(next) });
            unsafe { top.as_mut() }
        })
    }
}

pub struct SList<T: HasSingleLink<T=T>> {
    len: usize, 
    top: Option<Box<T>>,
}

impl<T: HasSingleLink<T=T>> SList<T> {
        
    pub fn new() -> SList<T> {
        SList::default() 
    }

    pub fn push(&mut self, mut new_head: Box<T>) {
        assert!(new_head.slink().link.is_none());
        new_head.slink_mut().link = self.top.take();
        self.top = Some(new_head);
    }

    pub fn pop(&mut self) -> Option<Box<T>> {
        self.top.take().map(|mut top| {
            self.top = top.slink_mut().link.take();
            top
        })
    }

    /// Remove the first node in the list that satisfies the given condition.
    pub fn remove_where<F: Fn(&T) -> bool>(&mut self, cond: F) -> Option<Box<T>> {
        self.top.take().and_then(|mut top| {
            if cond(&*top) {
                // The top element matched. Remove it and set the top to the next pointer.
                self.top = top.slink_mut().link.take();
                Some(top)
            } else {
                // The top element didn't match. Reset the top pointer and try to find the element
                // to be removed.
                self.top = Some(top);
                let mut res: Option<Box<T>> = None;
                for item in self {
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
                res
            }
        })
    }
}

impl<'a, T: HasSingleLink<T=T>> IntoIterator for &'a SList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        Iter { 
            top: self.top.as_ref().map(|top| &**top)
        }
    }
}

impl<'a, T: HasSingleLink<T=T>> IntoIterator for &'a mut SList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> {
        IterMut {
            top: self.top.as_mut().map(|top| unsafe { Raw::from_box(&mut *top) }),
            _marker: marker::PhantomData
        }
    }
}

impl<T: HasSingleLink<T=T>> Default for SList<T> {
    fn default() -> SList<T> {
        SList {
            len: 0,
            top: None,
        }
    }   
}
