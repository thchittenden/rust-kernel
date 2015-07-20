#![crate_name="loader"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate fs;

pub mod elf;

pub trait Loadable {
    
    fn load(

}
