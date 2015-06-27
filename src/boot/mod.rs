#![crate_name="boot"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude)]
#![no_std]
//!
//! This module is the entry point of the kernel. It is responsible for initializing all other
//! modules and beginning the first task.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate io;
extern crate interrupt;
extern crate alloc;
extern crate sched;
extern crate task;
extern crate mem;
extern crate devices;
extern crate collections;
extern crate fs;

mod test;

use util::multiboot::MultibootHeader;
use interrupt::{timer, BREAKPOINT_IRQ, Regs, IRet};
use task::thread::Thread;
logger_init!(Trace);

/// The kernel entry point. This should never return.
#[no_mangle]
pub extern fn kernel_main (hdr: &MultibootHeader) -> ! {
    println!(io::console::CON, "Booting kernel...");
    
    // Initialize IO (serial ports, etc.) This must be performed early as all logging may go
    // through COM1.
    io::init();
    
    // Initialize the interrupt subsystem. Install a no-op handler for breakpoints since apparently
    // they're added to rust code sometimes... See:
    // https://internals.rust-lang.org/t/attention-hackers-filling-drop/1715
    interrupt::init();
    interrupt::set_isr(BREAKPOINT_IRQ, nop);
    timer::set_frequency(19);

    // Initialize the allocator.
    alloc::init();

    // Initialize physical memory and virtual memory and enable paging.
    mem::init(hdr);
    
    // Create the root file system.
    fs::init();

    // Initialize all devices.
    devices::init();

    // Initialize the scheduler.
    sched::init();

    // Perform some self tests.
    test::test_all();

    // Do nothing.
    loop { }
    
    // Create some threads.
    let t1 = Thread::new(threadfn).unwrap();
    let t2 = Thread::new(threadfn).unwrap();
    sched::schedule_thread(t1);
    sched::schedule_thread(t2);
    sched::begin();
}

fn threadfn() -> ! {
    let tid = sched::get_tid();
    loop { trace!("hello from thread {}", tid) }
}

fn nop(_: u8, _: &mut Regs, _: &mut IRet) {
    trace!("breakpoint");
}



