#![crate_name="sched"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude)]
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
use collections::dlist::DList;
use task::thread::Thread;
use lock::SchedLock;
use interrupt::{pic, Regs, IRet, TIMER_INT_IRQ};

extern {
    /// Performs a context switch from one thread to another. While this claims to borrow them
    /// immutably, it lies and will in fact modify the `from` Thread's `stack_cur` field.
    fn context_switch(from: &Thread, to: &Thread);
    fn context_switch_first(to: &Thread) -> !;
}

struct Scheduler {
    thread: Option<Box<Thread>>,
    runnable: DList<Thread>,
}

static SCHED: SchedLock<Scheduler> = static_schedlock!(Scheduler {
    thread: None,
    runnable: DList { len: 0, head: None, tail: None } 
});

pub fn init() {
    interrupt::set_isr(TIMER_INT_IRQ, timer_interrupt);
}

// Begins the scheduler.
pub fn begin() -> ! {
    let mut s = SCHED.lock();
    assert!(s.thread.is_none());
    assert!(s.runnable.length() > 0);

    // TODO. Initialize idle here.

    // Put the first thread in the running position.
    let next_thread = s.runnable.pop_head().unwrap();
    s.thread = Some(next_thread);

    // Context switch to the new thread. TODO file bug? If I don't annotate the type of `thread`
    // then rustc fails with an error in LLVM codegen.
    let thread: &Thread = s.thread.as_ref().unwrap();
    unsafe { context_switch_first(thread) }
}

pub fn get_tid() -> i32 {
    SCHED.lock().thread.as_ref().unwrap().tid
}

// Apparently `yield` is reserved! Bah!
pub fn _yield (tid: Option<usize>) {
    let mut s = SCHED.lock(); 

    match tid {
        Some(_) => unimplemented!(),
        None => {
            // Move the current thread to the end of the queue and move the next thread into the running
            // thread position.
            let curr_thread = s.thread.take().unwrap();
            let next_thread = s.runnable.pop_head().unwrap();
            s.runnable.push_tail(curr_thread);
            s.thread = Some(next_thread);

            // Perform the stack swap.
            let curr_thread: &Thread = s.runnable.borrow_tail().unwrap();
            let next_thread: &Thread = s.thread.as_ref().unwrap();
            unsafe { context_switch(curr_thread, next_thread) };
        }
    }

}

fn timer_interrupt(id: u8, _: &mut Regs, _: &mut IRet) {
    let _ = SCHED.lock();

    // Once interrupts are disabled we can acknowledge the PIC. It's important to do this before
    // the context switch!
    pic::acknowledge_irq(id);

    // Yield to whoever's next.
    _yield(None);
}

pub fn schedule_thread(t: Box<Thread>) {
    let mut s = SCHED.lock();
    s.runnable.push_tail(t);
}



/// This is the mutex's interface to the scheduler.
#[no_mangle]
pub extern fn sched_yield(tid: Option<usize>) {
    _yield(tid)
}
