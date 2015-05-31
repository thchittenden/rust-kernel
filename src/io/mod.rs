#![crate_name="io"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]
//! 
//! This module contains interfaces for interacting with various IO components such as serial
//! ports and the keyboard.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate mutex;
extern crate interrupt;

/// The serial port module.
pub mod serial;
pub mod console;
pub mod keyboard;

use util::global::Global;
use serial::{SafeSerialPort, LCR_8N1};
use keyboard::keyboard_handler;
use interrupt::{set_isr, KEYBOARD_INT_IRQ};

/// The system-wide COM1 port.
pub static COM1: Global<SafeSerialPort> = global_init!();

/// Initializes all IO components.
pub fn init() {
    COM1.init(SafeSerialPort::new(0x3f8, 115200, LCR_8N1));
    set_isr(KEYBOARD_INT_IRQ, keyboard_handler);
}
