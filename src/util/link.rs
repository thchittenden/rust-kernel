//!
//! This module contains the definition of the `SingleLink` and `DoubleLink` objects used by 
//! various collections.
//! 
//! The `SingleLink` object owns the pointer to the next object. The `DoubleLink` object adds an
//! unsafe back pointer to the `SingleLink`.
//!
use core::prelude::*;
use raw::Raw;
use Pointer;

pub struct SingleLink<T: ?Sized, P: Pointer<To=T>> {
    pub link: Option<P>
}

pub struct DoubleLink<T: ?Sized, P: Pointer<To=T>> {
    pub next: SingleLink<T, P>,
    pub prev: Option<Raw<T>>
}

pub trait HasDoubleLink {
    /// The underlying type that contains the link.
    type T: ?Sized;

    /// The pointer type the link should contain.
    type P: Pointer<To=Self::T>;

    fn dlink(&self) -> &DoubleLink<Self::T, Self::P>;
    fn dlink_mut(&mut self) -> &mut DoubleLink<Self::T, Self::P>;
}

pub trait HasSingleLink {
    /// The underlying type that contains the link.
    type T: ?Sized;

    /// The pointer type the link should contain.
    type P: Pointer<To=Self::T>;

    fn slink(&self) -> &SingleLink<Self::T, Self::P>;
    fn slink_mut(&mut self) -> &mut SingleLink<Self::T, Self::P>;
}

impl<T: ?Sized, P: Pointer<To=T>> Default for SingleLink<T, P> {
    fn default() -> SingleLink<T, P> {
        SingleLink { link: None }
    }
}

impl<T: ?Sized, P: Pointer<To=T>> Default for DoubleLink<T, P> {
    fn default() -> DoubleLink<T, P> {
        DoubleLink { next: SingleLink::default(), prev: None }
    }
}

/// Any type that as a double link also has a single link.
impl<T: HasDoubleLink<T=T, P=P>, P: Pointer<To=T>> HasSingleLink for T {
    type T = T;
    type P = P;
    fn slink(&self) -> &SingleLink<T, P> {
        &self.dlink().next
    }
    fn slink_mut(&mut self) -> &mut SingleLink<T, P> {
        &mut self.dlink_mut().next
    }
}

