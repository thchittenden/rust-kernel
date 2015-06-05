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
#[macro_use] extern crate io;
extern crate interrupt;
extern crate alloc;
extern crate sched;
extern crate task;
extern crate mem;

use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
use core::fmt;

use util::multiboot::MultibootHeader;
use interrupt::{timer, BREAKPOINT_IRQ, Regs, IRet};
use task::thread::Thread;
logger_init!(Trace);

/// The kernel entry point. This should never return.
#[no_mangle]
pub extern fn kernel_main (hdr: &MultibootHeader) -> ! {
    println!(io::console::CON, "Booting kernel...");
    
    // Initialize the interrupt subsystem. Install a no-op handler for breakpoints since apparently
    // they're added to rust code sometimes... See:
    // https://internals.rust-lang.org/t/attention-hackers-filling-drop/1715
    interrupt::init();
    interrupt::set_isr(BREAKPOINT_IRQ, nop);
    timer::set_frequency(19);

    // Initialize IO (serial ports, etc.) This must be performed early as all logging may go
    // through COM1.
    io::init();

    // Initialize the allocator.
    alloc::init();

    // Initialize physical memory and virtual memory and enable paging.
    mem::init(hdr);
    
    // Initialize all devices.
    //devices::init();

    // Initialize the scheduler.
    sched::init();

    test_boxes();
    test_rc();

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




trait Foo {
    fn grok(&self) -> usize;
}

struct Bar { a: usize, b: isize }

impl Foo for Bar {
    fn grok(&self) -> usize { self.a + self.b as usize }
}

fn test_boxes() {

    trace!("\ntesting boxes");
    // Test recursive drops.
    let x = Box::new(3).unwrap();
    let y = Box::new(x).unwrap();
    trace!("got {}", y);
    trace!(" or {}", **y);
    drop(y);

    let z = unsafe { Box::new(4).unwrap().into_raw() };
    trace!("leaking {:p}", z);

    // Test unsized drops.
    let a = Bar { a: 1, b: 2 };
    let b = Box::new(a).unwrap();
    test_unsized(b);

}

struct Baz {
    rc: AtomicUsize,
    val: usize
}

impl HasRc for Baz {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}

impl fmt::Debug for Baz {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Baz {{ val: {:?} }}", self.val)
    }
}

fn test_rc() {

    trace!("\ntesting rc");
    let x = Box::new(Baz { rc: ATOMIC_USIZE_INIT, val: 4 }).unwrap(); 
    let rcx1 = Rc::new(x);
    let rcx2 = rcx1.clone();

    trace!("rcx1: {:?}", rcx1);
    trace!("rcx2: {:?}", rcx2);

    drop(rcx1);
    drop(rcx2);
}

fn test_unsized(a: Box<Foo>) {
    drop(a)
}
