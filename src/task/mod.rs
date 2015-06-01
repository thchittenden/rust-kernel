#![crate_name="task"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]
//!
//! This module contains definitions of task and thread structures.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;
extern crate collections;
extern crate mem;
extern crate io;

/// Thread related structures.
pub mod thread;

use mem::virt::PageDirectory;
use util::rawbox::RawBox;

#[repr(C, packed)]
pub struct Task {
 
    cr3: RawBox<PageDirectory>,

}

impl Task {
    
}
