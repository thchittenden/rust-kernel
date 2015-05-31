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
extern crate mem;
extern crate io;

use core::mem::drop;
use alloc::boxed::Box;
use util::multiboot::MultibootHeader;
use util::asm;
use interrupt::{pic, timer, TIMER_INT_IRQ};
use io::console::CON;
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

    // Enable interrupts for interrupt testing.
    asm::enable_interrupts();

    // Don't return.
    println!(CON, "Waiting for input...");
    loop { print!(CON, "{}", io::keyboard::getc()) }
}

fn timer_interrupt(id: u8, regs: &mut interrupt::Regs, iret: &mut interrupt::IRet) {
    trace!("timer interrupt");
    pic::acknowledge_irq(id);
}
