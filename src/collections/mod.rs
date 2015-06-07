#![crate_name="collections"]
#![crate_type="rlib"]
#![feature(no_std,core,filling_drop)]
#![no_std]
//!
//! This module contains definitions of various collections used in the kernel. 
//!
//! These collections generally follow the pattern of using an intrusive pointer in order to avoid
//! allocation. In order to support this, objects implement the `HasNode` trait which exposes a
//! reference to a `Node` embedded in the object. 
//!

#[macro_use] extern crate core;
extern crate util;
extern crate alloc;

/// A dynamically resizable array.
pub mod dynarray;

/// An owning linked list.
pub mod linkedlist;

/// A separately-chained hash map.
//pub mod hashmap;

/// A vector.
pub mod vec;
