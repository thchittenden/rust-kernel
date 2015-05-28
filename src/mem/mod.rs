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

use core::prelude::*;
use alloc::boxed::Box;
use rawbox::RawBox;
use virt::{PageTableEntry, PageDirectoryEntry, PageTable, PageDirectory};
use virt::{PDE_WRITABLE, PDE_SUPERVISOR, PDE_MAPPED_SIZE};
use util::{page_align, is_page_aligned, PAGE_SIZE};
use util::multiboot::MultibootHeader;
use util::asm::enable_paging;
logger_init!(Trace);

// The kernel page directory. This is the default page directory used by new tasks. 
static kpd: Option<RawBox<PageDirectory>> = None;

pub fn init(hdr: &MultibootHeader) {
    phys::init();
    virt::init();
    hdr.walk_mmap(add_range_safe);
    direct_map_kernel(); 
    //enable_paging();
}

fn direct_map_kernel() {
    trace!("direct mapping kernel");
    let mut pd = PageDirectory::new().unwrap();
    let mut pte0 = PageTable::new().unwrap();
    let mut pte1 = PageTable::new().unwrap();
    let mut pte2 = PageTable::new().unwrap();
    let mut pte3 = PageTable::new().unwrap();
    trace!("pd: {:?}", pd);
    trace!("pte0: {:?}", pte0);
    trace!("pte1: {:?}", pte1);
    trace!("pte2: {:?}", pte2);
    trace!("pte3: {:?}", pte3);

    // First, map the page directory into itself. This is ok because page directories look a lot
    // like page tables so by mapping the page directory into itself causes that entry to in the
    // page directory to map all page tables. See the following link if interested.
    // http://wiki.osdev.org/Page_Tables#Recursive_mapping
    let pd_pt = unsafe { pd.as_pagetable() }; //FIXME: RFX/811
    pd.pdes[1023].set_pagetable(pd_pt);

    // Map in the four page tables.
    let pdflags = PDE_SUPERVISOR | PDE_WRITABLE;
    pd.map_pagetable(0*PDE_MAPPED_SIZE, pte0, pdflags);
    pd.map_pagetable(1*PDE_MAPPED_SIZE, pte1, pdflags);
    pd.map_pagetable(2*PDE_MAPPED_SIZE, pte2, pdflags);
    pd.map_pagetable(3*PDE_MAPPED_SIZE, pte3, pdflags);

    // Map in the kernel.
    let kernel_start = linker_sym!(__kernel_start);
    let kernel_end = linker_sym!(__kernel_end);
    for page in (kernel_start..kernel_end).step_by(PAGE_SIZE) {
        assert!(pd.has_pagetable(page));
    }

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

