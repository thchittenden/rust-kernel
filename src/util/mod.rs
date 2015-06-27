//! This crate contains a collection of dependent-less utility functions and constants used
//! throughout the kernel.
//!
#![crate_name="util"]
#![crate_type="rlib"]
#![feature(no_std,core,asm,unique,core_prelude,const_fn)]
#![no_std]

#[macro_use] extern crate core;

/// Utility macros.
#[macro_use] pub mod macros;

/// Assembly wrappers.
pub mod asm;

/// The bitflags library.
pub mod bitflags;

/// A wrapper for init-once globals.
pub mod global;

/// Logging macros.
pub mod logger;

/// Multiboot header struct.
pub mod multiboot;

/// A box type for nonmanaged pointers.
pub mod rawbox;

/// The kernel result type.
pub mod kernresult;

pub use kernresult::KernResult;
pub use kernresult::KernError;
pub use kernresult::KernResultEx;
pub use kernresult::KernErrorEx;

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

/// Returns whether an address is page aligned or not.
#[inline]
pub fn is_page_aligned(addr: usize) -> bool {
    is_aligned!(addr, PAGE_SIZE)
}

/// Aligns an address to the page it's contained in.
#[inline]
pub fn page_align(addr: usize) -> usize {
    align!(addr, PAGE_SIZE)
}

/// Returns the number of low order 0 bits in an alignment mask. This is currently used in the LMM
/// allocator.
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

/// Returns whether a value is a power of two or not.
#[inline]
pub fn is_pow2(val: usize) -> bool {
    val & (val - 1) == 0
}

