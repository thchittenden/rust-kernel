#![crate_name="collections"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
extern crate alloc;

#[macro_use] pub mod queue;
pub mod node;

