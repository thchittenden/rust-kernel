#![feature(no_std,core)]
#![no_std]

extern crate core;

pub use console::Console;
pub use color::*; 

pub mod console;
pub mod color;

