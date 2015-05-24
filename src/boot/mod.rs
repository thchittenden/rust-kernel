#![feature(no_std,core)]
#![no_std]

extern crate core;

mod multiboot;
use multiboot::MultibootHeader;

#[no_mangle]
pub fn kernel_main (multiboot_magic: u32, multiboot_header: *const MultibootHeader) {



}
