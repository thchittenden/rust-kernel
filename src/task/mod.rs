#![crate_name="task"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude)]
#![no_std]
//!
//! This module contains definitions of task and thread structures.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate io;
extern crate alloc;
extern crate collections;
extern crate mem;

/// Thread related structures.
pub mod thread;

use mem::virt::AddressSpace;

pub struct Task {

    addrspace: AddressSpace,

}

impl Task {
    
}
