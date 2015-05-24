#![crate_name="console"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate sync;

use console::SAFE_CONSOLE_INIT;

pub use console::SafeConsole;
pub use console::Console;
pub use color::*; 

pub mod console;
pub mod color;

pub static CON: SafeConsole = SAFE_CONSOLE_INIT;

