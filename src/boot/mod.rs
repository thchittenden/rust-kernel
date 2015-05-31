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
extern crate interrupt;
extern crate alloc;
extern crate sched;
extern crate task;
extern crate mem;
extern crate io;

use core::mem::drop;
use alloc::boxed::Box;
use util::multiboot::MultibootHeader;
use util::asm;
use interrupt::{pic, timer, TIMER_INT_IRQ};
use io::console::CON;
use task::thread::Thread;
logger_init!(Trace);

/// The kernel entry point. This should never return.
#[no_mangle]
pub extern fn kernel_main (hdr: &MultibootHeader) -> ! {
    println!(CON, "Booting kernel...");
    
    // Initialize the interrupt subsystem.
    interrupt::init();
    timer::set_frequency(19);

    // Initialize IO (serial ports, etc.) This must be performed early as all logging may go
    // through COM1.
    io::init();

    // Initialize the allocator.
    alloc::init();

    // Initialize physical memory with all valid memory regions.
    mem::init(hdr);

    // Initialize the scheduler.
    sched::init();

    // Create some threads.
    let t1 = Thread::new(|| { }).unwrap();
    let t2 = Thread::new(|| { }).unwrap();
    sched::schedule_thread(t1);
    sched::schedule_thread(t2);
    sched::begin();
}

