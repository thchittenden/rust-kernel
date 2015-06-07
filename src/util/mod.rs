#![crate_name="util"]
#![crate_type="rlib"]
#![feature(no_std,core,asm,unique)]
#![no_std]

#[macro_use] extern crate core;

#[macro_use] pub mod macros;
pub mod asm;
pub mod bitflags;
pub mod global;
pub mod logger;
pub mod multiboot;
pub mod rawbox;
pub mod link;
pub mod raw;

use core::prelude::*;

pub const NULL_SEGMENT: u16 = 0x0000;
pub const KERNEL_CODE_SEGMENT: u16 = 0x0008;
pub const KERNEL_DATA_SEGMENT: u16 = 0x0010;
pub const USER_CODE_SEGMENT: u16 = 0x0018;
pub const USER_DATA_SEGMENT: u16 = 0x0020;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SHIFT: usize = 12;

/// A trait for indicating that a type points to another type.
pub trait Pointer { 

    /// The type this pointer points to.
    type To: ?Sized;

    /// Get the pointer to the underlying object.
    fn as_ref(&self) -> &Self::To;

    /// Get a mutable pointer to the underlying object.
    fn as_mut(&mut self) -> &mut Self::To;
}

#[macro_export]
macro_rules! getbyte {
    ($val:expr, $byte:expr) => { ($val >> (8 * $byte)) as u8 };
}

#[inline]
pub fn is_page_aligned(addr: usize) -> bool {
    is_aligned!(addr, PAGE_SIZE)
}

#[inline]
pub fn page_align(addr: usize) -> usize {
    align!(addr, PAGE_SIZE)
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

