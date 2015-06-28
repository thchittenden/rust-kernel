#![crate_name="collections"]
#![crate_type="rlib"]
#![feature(no_std,core,filling_drop,core_prelude,core_str_ext,core_intrinsics,const_fn,core_char_ext)]
#![no_std]
//!
//! This module contains definitions of various collections used in the kernel. 
//!
//! These collections generally follow the pattern of using an intrusive pointer in order to avoid
//! allocation. In order to support this, objects implement the `HasNode` trait which exposes a
//! reference to a `Node` embedded in the object. 
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;

/// An aliasable mutable pointer. This completely subverts all safety provided by Rust but makes it
/// a lot more convenient to deal with circular data structures! It should not be exported as Raw
/// pointers should NEVER escape this module.
mod raw;

/// A link typed used by various collections.
pub mod link;

/// A wrapper for any type to add a link to it.
pub mod linked;

/// A dynamically resizable array.
pub mod dynarray;

/// A singly linked list.
pub mod slist;

/// An doubly linked list.
pub mod dlist;

/// A separately-chained hash map.
pub mod hashmap;

/// A vector.
pub mod vec;

/// A uniform interface to both dynamic and static strings.
pub mod string;
