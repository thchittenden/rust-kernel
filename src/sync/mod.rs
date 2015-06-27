#![crate_name="sync"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude,const_fn)]
#![no_std]

#[macro_use] extern crate core;
extern crate collections;
extern crate mutex;
extern crate task;
extern crate alloc;

pub mod condvar;
pub mod semaphore;
pub mod rwlock;

