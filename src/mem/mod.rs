#![crate_name="mem"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate macros;
#[macro_use] extern crate sync;
extern crate console;
extern crate util;

pub mod phys;
pub mod virt;

pub fn init() {
    phys::init();
}
