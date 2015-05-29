#![crate_name="boot"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]
//!
//! This module is the entry point of the kernel. It is responsible for initializing all other
//! modules and beginning the first task.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate console;
extern crate mem;
extern crate alloc;
extern crate io;

use core::mem::drop;
use alloc::boxed::Box;
use util::multiboot::MultibootHeader;
use mem::phys;
logger_init!(Trace);

/// The kernel entry point. This should never return.
#[no_mangle]
pub extern "C" fn kernel_main (hdr: &MultibootHeader) -> ! {
    
    // Initialize IO (serial ports, etc.) This must be performed first as all logging
    // functions may go to COM1.
    io::init();

    // Initialize the allocator.
    alloc::init();

    // Initialize physical memory with all valid memory regions.
    mem::init(hdr);
    
    let x = Box::new(3);
    let y = Box::new(x);
    trace!("y: {:?}", y);
    drop(y);

    // Don't return.
    loop { }
}

