#![crate_name="mem"]
#![crate_type="rlib"]
#![feature(no_std,core,step_by)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate sync;
extern crate console;

pub mod phys;
pub mod virt;

pub fn init() {
    phys::init();
}
