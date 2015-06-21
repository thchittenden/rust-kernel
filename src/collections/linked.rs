use core::prelude::*;
use core::ops::{Deref, DerefMut};
use link::{DoubleLink, HasDoubleLink};

pub struct Linked<T: ?Sized> {
    link: DoubleLink<Linked<T>>,
    value: T,
}

impl<T> Linked<T> {
    pub fn new(value: T) -> Linked<T> {
        Linked {
            value: value,
            link: DoubleLink::default(),
        }
    }
}

impl<T: ?Sized> HasDoubleLink<Linked<T>> for Linked<T> {
    fn dlink(&self) -> &DoubleLink<Linked<T>> {
        &self.link
    }
    fn dlink_mut(&mut self) -> &mut DoubleLink<Linked<T>> {
        &mut self.link
    }
}

impl<T: ?Sized> Deref for Linked<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: ?Sized> DerefMut for Linked<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl <T: Clone + ?Sized> Clone for Linked<T> {
    fn clone(&self) -> Linked<T> {
        Linked {
            value: self.value.clone(),
            link: DoubleLink::default(),
        }
    }
}
