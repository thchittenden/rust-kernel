#![crate_name="mem"]
#![crate_type="rlib"]
#![feature(no_std,core,step_by,negate_unsigned,core_prelude,core_intrinsics,result_expect,const_fn)]
#![no_std]
//!
//! This module contains definitions for interacting with physical/virtual memory.
//! 

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate io;
extern crate alloc;
extern crate mutex;
extern crate sync;

pub mod phys;
pub mod virt;

use core::prelude::*;
use phys::{Frame, FrameReserve};
use virt::AddressSpace;
use util::{page_align, PAGE_SIZE};
use util::global::Global;
use util::multiboot::MultibootHeader;
use util::asm::{enable_paging, enable_global_pages};
logger_init!(Trace);

// The kernel address space. This is the default address space for all new tasks.
static KAS: Global<AddressSpace> = Global::new();

/// Initializes all memory related submodules. 
///
/// This uses the `MultibootHeader` to populate the free frame list with all free physical frames.
/// It then uses 5 frames to direct map the first 16 MB of the address space, which is reserved for
/// the kernel. Finally, it enables paging.
pub fn init(hdr: &MultibootHeader) {
    debug!("initializing phys mem");
    phys::init();
    hdr.walk_mmap(add_range_safe);
    
    debug!("initializing virt mem");
    let addrspace = virt::init();
    KAS.init(addrspace);

    // Notify physical memory module about paging.
    phys::enable_paging(&KAS);
}

// This function filters memory ranges reported by the bootloader to remove the
// pages reserved for kernel memory.
fn add_range_safe(region_start: usize, region_end: usize) {
    let kernel_start: usize = linker_sym!(__kernel_start);
    let kernel_end: usize = linker_sym!(__kernel_end);
    let region_end = page_align(region_end);
    if region_start < kernel_start && region_end > kernel_start {
        // Region overlaps from the left. 
        if region_end > kernel_end {
            // Region extends past the kernel, split it.
            phys::add_range(region_start, kernel_start);
            phys::add_range(kernel_end, region_end);
        } else {
            // Region does not extend past kernel, chop off the right.
            phys::add_range(region_start, kernel_start);
        }
    } else if region_start >= kernel_start && region_start < kernel_end {
        // Region overlaps from the middle. Chop off the left. 
        phys::add_range(kernel_end, region_end);
    } else {
        // No overlap.
        phys::add_range(region_start, region_end);
    }
}

