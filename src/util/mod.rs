#![crate_name="util"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
extern crate console;

pub mod multiboot;
pub mod macros;
pub mod logger;
pub mod bitflags;

pub const PAGE_SIZE: usize = 0x1000;

#[inline]
pub fn is_page_aligned(addr: usize) -> bool {
    is_aligned(addr, PAGE_SIZE)
}

#[inline]
pub fn page_align(addr: usize) -> usize {
    align(addr, PAGE_SIZE)
}

pub fn is_aligned(addr: usize, alignment: usize) -> bool {
    assert!(is_pow2(alignment));
    (addr & (alignment - 1)) == 0
}

#[inline]
pub fn align(addr: usize, alignment: usize) -> usize {
    assert!(is_pow2(alignment));
    (addr & !(alignment - 1))
}

#[inline]
pub fn is_pow2(val: usize) -> bool {
    val & (val - 1) == 0
}
