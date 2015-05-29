#![crate_name="sched"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;

use core::prelude::*;

pub fn init() {

}

// Begins the scheduler.
pub fn begin() -> ! {
    unimplemented!()
}

/// This is the mutex's interface to the scheduler.
#[no_mangle]
pub extern fn sched_yield(tid: Option<usize>) {
    unimplemented!()
}
