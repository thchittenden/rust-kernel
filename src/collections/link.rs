//!
//! This module contains the definition of the `SingleLink` and `DoubleLink` objects used by 
//! various collections.
//! 
//! The `SingleLink` object owns the pointer to the next object. The `DoubleLink` object adds an
//! unsafe back pointer to the `SingleLink`.
//!
use alloc::boxed::Box;
use core::prelude::*;
use raw::Raw;

pub struct SingleLink<T: ?Sized> {
    pub link: Option<Box<T>>
}

pub struct DoubleLink<T: ?Sized> {
    pub next: SingleLink<T>,
    pub prev: Option<Raw<T>>
}

pub trait HasDoubleLink {
    /// The underlying type that contains the link.
    type T: ?Sized;
    fn dlink(&self) -> &DoubleLink<Self::T>;
    fn dlink_mut(&mut self) -> &mut DoubleLink<Self::T>;
}

pub trait HasSingleLink {
    /// The underlying type that contains the link.
    type T: ?Sized;
    fn slink(&self) -> &SingleLink<Self::T>;
    fn slink_mut(&mut self) -> &mut SingleLink<Self::T>;
}

impl<T: ?Sized> Default for SingleLink<T> {
    fn default() -> SingleLink<T> {
        SingleLink { link: None }
    }
}

impl<T: ?Sized> Default for DoubleLink<T> {
    fn default() -> DoubleLink<T> {
        DoubleLink { next: SingleLink::default(), prev: None }
    }
}

/// Any type that as a double link also has a single link.
impl<T: HasDoubleLink<T=T>> HasSingleLink for T {
    type T = T;
    fn slink(&self) -> &SingleLink<T> {
        &self.dlink().next
    }
    fn slink_mut(&mut self) -> &mut SingleLink<T> {
        &mut self.dlink_mut().next
    }
}

