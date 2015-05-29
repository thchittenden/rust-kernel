#![crate_name="console"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]
//!
//! This module contains an interface for interacting with the VGA console. All operations are
//! performed through a `SafeConsole` which uses a mutex to protect a regular `Console`.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate mutex;

use console::SAFE_CONSOLE_INIT;

pub use console::SafeConsole;
pub use console::Console;
pub use color::*; 

/// The interface to the VGA console.
pub mod console;

/// Definitions of the colors used in the console.
pub mod color;

/// The system-wide console.
pub static CON: SafeConsole = SAFE_CONSOLE_INIT;

