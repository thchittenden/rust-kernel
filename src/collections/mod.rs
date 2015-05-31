#![crate_name="collections"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]
//!
//! This module contains definitions of various collections used in the kernel. 
//!
//! These collections generally follow the pattern of using an intrusive pointer in order to avoid
//! allocation. In order to support this, objects implement the `HasNode` trait which exposes a
//! reference to a `Node` embedded in the object. 
//!

#[macro_use] extern crate core;
extern crate alloc;

/// The `Node` object used by various collections.
pub mod node;

/// The `Raw` pointer that allows circular references in collections. This should absolutely not be
/// public as it circumvents all uniqueness guarantees for `Box`!
mod raw;

/// A linked list interface.
pub mod linkedlist;

