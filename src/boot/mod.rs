#![crate_name="boot"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate console;
extern crate mem;
extern crate alloc;

mod multiboot;

use alloc::boxed::Box;
use multiboot::MultibootHeader;
use mem::phys;

logger_init!(Trace);

#[no_mangle]
pub extern "C" fn kernel_main (hdr: &MultibootHeader) -> ! {
    trace!("hello from a brand new kernel");

    // Initialize the allocator.
    alloc::init();

    // Initialize physical memory with all valid memory regions.
    hdr.walk_mmap(add_range_safe);
    
    // Try some allocations.
    let x = Box::new(3);
    trace!("x: {:?}", x);
    //let y = Box::new(x);
    //trace!("y: {:?}", y);

    // Don't return.
    loop { }
}

// This function filters memory ranges reported by the bootloader to remove the
// pages reserved for kernel memory.
fn add_range_safe(region_start: usize, region_end: usize) {
    let kernel_start: usize = linker_sym!(__kernel_start);
    let kernel_end: usize = linker_sym!(__kernel_end);
    let region_end = util::page_align(region_end);
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
