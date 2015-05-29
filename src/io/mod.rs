#![crate_name="io"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate mutex;

pub mod serial;

use util::global::Global;
use serial::{SafeSerialPort, LCR_8N1};

pub static COM1: Global<SafeSerialPort> = global_init!();

pub fn init() {
    COM1.init(SafeSerialPort::new(0x3f8, 115200, LCR_8N1));
}
