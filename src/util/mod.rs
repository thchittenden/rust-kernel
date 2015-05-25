#![crate_name="util"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

extern crate core;
extern crate console;

pub mod macros;
pub mod logger;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_ALIGN_MASK: usize = PAGE_SIZE - 1;

#[inline]
pub fn is_page_aligned(addr: usize) -> bool {
    (addr & PAGE_ALIGN_MASK) == 0
}

#[inline]
pub fn page_align(addr: usize) -> usize {
    addr & !PAGE_ALIGN_MASK
}
