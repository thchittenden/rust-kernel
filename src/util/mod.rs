#![crate_name="util"]
#![crate_type="rlib"]
#![feature(no_std,core,asm,unboxed_closures,optin_builtin_traits)]
#![no_std]

#[macro_use] extern crate core;
extern crate console;

pub mod asm;
pub mod bitflags;
pub mod global;
pub mod logger;
pub mod macros;
pub mod multiboot;

pub const NULL_SEGMENT: u16 = 0x0000;
pub const KERNEL_CODE_SEGMENT: u16 = 0x0008;
pub const KERNEL_DATA_SEGMENT: u16 = 0x0010;
pub const USER_CODE_SEGMENT: u16 = 0x0018;
pub const USER_DATA_SEGMENT: u16 = 0x0020;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SHIFT: usize = 12;

#[macro_export]
macro_rules! getbyte {
    ($val:expr, $byte:expr) => { ($val >> (8 * $byte)) as u8 };
}

#[inline]
pub fn is_page_aligned(addr: usize) -> bool {
    is_aligned(addr, PAGE_SIZE)
}

#[inline]
pub fn page_align(addr: usize) -> usize {
    align(addr, PAGE_SIZE)
}

#[inline]
pub fn is_aligned(addr: usize, alignment: usize) -> bool {
    assert!(is_pow2(alignment));
    addr & (alignment - 1) == 0
}

#[inline]
pub fn align(addr: usize, alignment: usize) -> usize {
    assert!(is_pow2(alignment));
    addr & !(alignment - 1)
}

#[inline]
pub fn align_up(addr: usize, alignment: usize) -> usize {
    //assert!(is_pow2(alignment));
    (addr + alignment - 1) & !(alignment - 1)
}

#[inline]
pub fn align_bits(mut alignment: usize) -> usize {
    // We could use an instruction to do this more efficiently.
    let mut bits = 0;
    while alignment != 0 {
        alignment >>= 1;
        bits += 1;
    }
    bits
}

#[inline]
pub fn is_pow2(val: usize) -> bool {
    val & (val - 1) == 0
}

