#![crate_name="task"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
extern crate mem;

use core::ops::Fn;
use mem::rawbox::RawBox;
use mem::virt::PageDirectory;

#[repr(C, packed)]
struct Task {
 
    tid: i32,
    cr3: RawBox<PageDirectory>,

    stack_cur: usize, 
    stack_top: usize,
    stack_bottom: usize, // This MUST be at offset 0x10
}

impl Task {
    
    // Creates a new task that will execute the given function. Unfortunately
    // the type system does not allow diverging closures, so we must settle for
    // a closure that we promise will vanish when it's done.
    pub fn new<F> (taskfn: F) -> Task where F: Fn() -> () {
       unimplemented!() 
    }

}
