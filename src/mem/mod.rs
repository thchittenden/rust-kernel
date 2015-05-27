#![crate_name="mem"]
#![crate_type="rlib"]
#![feature(no_std,core,step_by,unique)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate sync;
extern crate alloc;
extern crate console;

pub mod rawbox;
pub mod phys;
pub mod virt;

use virt::{PageTable, PageTableEntry};
use util::{page_align, is_page_aligned, PAGE_SIZE};
use util::multiboot::MultibootHeader;
use util::asm::enable_paging;

pub fn init(hdr: &MultibootHeader) {
    phys::init();
    virt::init();
    hdr.walk_mmap(add_range_safe);
    direct_map_kernel(); 
    //enable_paging();
}

fn direct_map_kernel() {
     
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

