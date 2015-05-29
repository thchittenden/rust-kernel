#![crate_name="io"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]
//! 
//! This module contains interfaces for interacting with various IO components such as serial
//! ports.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate mutex;

/// The serial port module.
pub mod serial;

use util::global::Global;
use serial::{SafeSerialPort, LCR_8N1};

/// The system-wide COM1 port.
pub static COM1: Global<SafeSerialPort> = global_init!();

/// Initializes all IO components.
pub fn init() {
    COM1.init(SafeSerialPort::new(0x3f8, 115200, LCR_8N1));
}
