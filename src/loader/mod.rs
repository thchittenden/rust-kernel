#![crate_name="loader"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;
extern crate collections;
extern crate fs;
extern crate mem;

pub mod elf;

use mem::addrspace::AddressSpace;
use util::KernResult;

pub trait Loadable {
   
    /// Loads the executable into the address space.
    fn load(&self, addrspace: &mut AddressSpace) -> KernResult<()>;
    
    /// Gets the entry point to the image.
    fn entry(&self) -> KernResult<usize>;

}
