#![crate_name="sync"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate collections;
#[macro_use] extern crate mutex;
#[macro_use] extern crate task;
extern crate alloc;

#[macro_use] pub mod condvar;
#[macro_use] pub mod semaphore;

