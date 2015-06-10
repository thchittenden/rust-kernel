use core::prelude::*;
use core::ops::{Deref, DerefMut};
use alloc::rc::Rc;
use link::{DoubleLink, HasDoubleLink};

struct Linked<T> {
    value: T,
    link: DoubleLink<Linked<T>>,
}

impl<T> Linked<T> {
    pub fn new(value: T) -> Linked<T> {
        Linked {
            value: value,
            link: DoubleLink::default(),
        }
    }
}

impl<T> HasDoubleLink for Linked<T> {
    type T = Linked<T>;
    fn dlink(&self) -> &DoubleLink<Self::T> {
        &self.link
    }

    fn dlink_mut(&mut self) -> &mut DoubleLink<Self::T> {
        &mut self.link
    }
}

impl<T> Deref for Linked<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Linked<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl <T: Clone> Clone for Linked<T> {
    fn clone(&self) -> Linked<T> {
        Linked {
            value: self.value.clone(),
            link: DoubleLink::default(),
        }
    }
}
