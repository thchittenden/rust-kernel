#![crate_name="sched"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate collections;
#[macro_use] extern crate util;
extern crate interrupt;
extern crate alloc;
extern crate task;
extern crate io;

#[macro_use] mod lock;

use core::prelude::*;
use alloc::boxed::Box;
use collections::linkedlist::LinkedList;
use collections::node::HasNode;
use task::thread::Thread;
use lock::SchedLock;
use interrupt::{pic, Regs, IRet, TIMER_INT_IRQ};
logger_init!(Trace);

struct Scheduler {
    thread: Option<Box<Thread>>,
    runnable: LinkedList<Thread>,
}

static SCHED: SchedLock<Scheduler> = static_schedlock!(Scheduler {
    thread: None,
    runnable: LinkedList { head: None, tail: None } 
});

pub fn init() {
    interrupt::set_isr(TIMER_INT_IRQ, timer_interrupt);
}

fn timer_interrupt(id: u8, regs: &mut Regs, iret: &mut IRet) {
    trace!("timer interrupt in sched");  
    pic::acknowledge_irq(id);
}

// Begins the scheduler.
pub fn begin() -> ! {
    unimplemented!()
}

pub fn schedule_thread(t: Box<Thread>) {
    let mut s = SCHED.lock();
    s.runnable.push_tail(t);
}



/// This is the mutex's interface to the scheduler.
#[no_mangle]
pub extern fn sched_yield(tid: Option<usize>) {
    unimplemented!()
}
