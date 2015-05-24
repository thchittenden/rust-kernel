#![crate_name="boot"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate macros;
extern crate console;

mod multiboot;

use multiboot::MultibootHeader;
use console::Console;
use core::option::Option::None;

#[no_mangle]
pub extern "C" fn kernel_main (multiboot_magic: u32, multiboot_header: *const MultibootHeader) {
    let mut con = Console::new();
    println!(con, "hi");
    println!(con, "test {:x}", multiboot_magic);
    let x = None;
    let y: () = x.unwrap();
}
