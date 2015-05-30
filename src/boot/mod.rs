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
extern crate interrupt;

use core::mem::drop;
use console::CON;
use alloc::boxed::Box;
use util::multiboot::MultibootHeader;
use util::asm;
use interrupt::{pic, timer, TIMER_INT_IRQ};
logger_init!(Trace);

/// The kernel entry point. This should never return.
#[no_mangle]
pub extern fn kernel_main (hdr: &MultibootHeader) -> ! {
    println!(CON, "Booting kernel...");
    
    // Initialize the interrupt subsystem.
    interrupt::init();
    interrupt::set_isr(TIMER_INT_IRQ, timer_interrupt);
    timer::set_frequency(19);

    // Initialize IO (serial ports, etc.) This must be performed early as all logging may go
    // through COM1.
    io::init();

    // Initialize the allocator.
    alloc::init();

    // Initialize physical memory with all valid memory regions.
    mem::init(hdr);
    
    let x = Box::new(3);
    let y = Box::new(x);
    trace!("y: {:?}", y);
    drop(y);

    // Enable interrupts for interrupt testing.
    asm::enable_interrupts();

    // Don't return.
    loop { }
}

fn timer_interrupt(id: u8, regs: &mut interrupt::Regs, iret: &mut interrupt::IRet) {
    trace!("timer interrupt");
    pic::acknowledge_irq(id);
}
