//!
//! This module contains the definition of the `SingleLink` and `DoubleLink` objects used by 
//! various collections.
//! 
//! The `SingleLink` object owns the pointer to the next object. The `DoubleLink` object adds an
//! unsafe back pointer to the `SingleLink`.
//!
use alloc::boxed::Box;
use core::prelude::*;
use core::fmt;
use raw::Raw;

pub struct SingleLink<T: ?Sized> {
    pub link: Option<Box<T>>
}

impl<T: ?Sized> SingleLink<T> {
    pub const fn new() -> SingleLink<T> {
        SingleLink { link: None }
    }
}

pub struct DoubleLink<T: ?Sized> {
    pub next: SingleLink<T>,
    pub prev: Option<Raw<T>>
}

impl<T: ?Sized> DoubleLink<T> {
    pub const fn new() -> DoubleLink<T> {
        DoubleLink {
            next: SingleLink::new(),
            prev: None,
        }
    }
}

pub trait HasSingleLink<T: ?Sized> {
    fn slink(&self) -> &SingleLink<T>;
    fn slink_mut(&mut self) -> &mut SingleLink<T>;
}

pub trait HasDoubleLink<T: ?Sized> {
    fn dlink(&self) -> &DoubleLink<T>;
    fn dlink_mut(&mut self) -> &mut DoubleLink<T>;
}

impl<T: ?Sized> Default for SingleLink<T> {
    fn default() -> SingleLink<T> {
        SingleLink::new()
    }
}

impl<T: ?Sized> Default for DoubleLink<T> {
    fn default() -> DoubleLink<T> {
        DoubleLink::new()
    }
}

/// Any type that as a double link also has a single link.
impl<T: HasDoubleLink<T> + ?Sized> HasSingleLink<T> for T {
    fn slink(&self) -> &SingleLink<T> {
        &self.dlink().next
    }
    fn slink_mut(&mut self) -> &mut SingleLink<T> {
        &mut self.dlink_mut().next
    }
}

impl<T> fmt::Debug for SingleLink<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.link.is_none() {
            write!(f, "EmptyLink")
        } else {
            write!(f, "FullLink")
        }
    }
}

impl<T> fmt::Debug for DoubleLink<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.next.link.is_none() && self.prev.is_none() {
            write!(f, "EmptyLink")
        } else {
            write!(f, "FullLink")
        }
    }
}


