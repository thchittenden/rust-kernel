#![crate_name="boot"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

extern crate core;
#[macro_use]
extern crate macros;
extern crate console;

mod multiboot;

use multiboot::MultibootHeader;
use console::Console;

#[no_mangle]
pub fn kernel_main (multiboot_magic: u32, multiboot_header: *const MultibootHeader) -> ! {
    let mut con = Console::new();
    println!(con, "hi");
    loop { }
}
