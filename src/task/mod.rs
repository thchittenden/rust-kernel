#![crate_name="task"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]
//!
//! This module contains definitions of task and thread structures.
//!

#[macro_use] extern crate core;
extern crate util;
extern crate collections;
extern crate mem;

/// Thread related structures.
pub mod thread;

use core::prelude::*;
use mem::virt::PageDirectory;
use util::rawbox::RawBox;

#[repr(C, packed)]
pub struct Task {
 
    cr3: RawBox<PageDirectory>,

}

impl Task {
    
    // Creates a new task that will execute the given function. Unfortunately
    // the type system does not allow diverging closures, so we must settle for
    // a closure that we promise will vanish when it's done.
    pub fn new<F> (taskfn: F) -> Option<Task> where F: Fn() -> () {

       unimplemented!() 
    }

}
