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
pub mod addrspace;

use core::prelude::*;
use phys::{Frame, FrameReserve};
use virt::{PageTable, PageDirectory};
use virt::{PDE_WRITABLE, PDE_SUPERVISOR, PDE_MAPPED_SIZE, PD_RECMAP_ADDR};
use virt::{PTE_WRITABLE, PTE_SUPERVISOR, PTE_GLOBAL};
use util::{page_align, PAGE_SIZE};
use util::rawbox::RawBox;
use util::global::Global;
use util::multiboot::MultibootHeader;
use util::asm::{enable_paging, enable_global_pages};
logger_init!(Trace);

// The kernel page directory. This is the default page directory used by new tasks. 
static KPD: Global<Frame<PageDirectory>> = Global::new();

/// Initializes all memory related submodules. 
///
/// This uses the `MultibootHeader` to populate the free frame list with all free physical frames.
/// It then uses 5 frames to direct map the first 16 MB of the address space, which is reserved for
/// the kernel. Finally, it enables paging.
pub fn init(hdr: &MultibootHeader) {
    debug!("initializing mem");
    phys::init();
    virt::init();
    hdr.walk_mmap(add_range_safe);
    
    trace!("direct mapping kernel");
    direct_map_kernel(); 

    trace!("enabling paging...");
    KPD.activate();
    enable_global_pages();
    enable_paging();
}

fn direct_map_kernel() {
    // Reserve the necessary frames.
    let resv = FrameReserve::new();
    resv.reserve(5).expect("unable to allocate initial kernel frames");

    // Get the frames.
    let mut pd = PageDirectory::new(resv.get_frame());
    let pt0 = PageTable::new(resv.get_frame());
    let pt1 = PageTable::new(resv.get_frame());
    let pt2 = PageTable::new(resv.get_frame());
    let pt3 = PageTable::new(resv.get_frame());
    trace!("pd: {:?}", pd);
    trace!("pt0: {:?}", pt0);
    trace!("pt1: {:?}", pt1);
    trace!("pt2: {:?}", pt2);
    trace!("pt3: {:?}", pt3);

    // First, map the page directory into itself. This is ok because page directories look a lot
    // like page tables so by mapping the page directory into itself causes that entry to in the
    // page directory to map all page tables. See the following link if interested.
    // http://wiki.osdev.org/Page_Tables#Recursive_mapping
    let pdflags = PDE_SUPERVISOR | PDE_WRITABLE;
    let pdrec = unsafe { pd.as_pagetable() }; //FIXME: RFC/811
    pd.map_pagetable(PD_RECMAP_ADDR, pdrec, pdflags);

    // Map in the four page tables.
    pd.map_pagetable(0*PDE_MAPPED_SIZE, pt0, pdflags);
    pd.map_pagetable(1*PDE_MAPPED_SIZE, pt1, pdflags);
    pd.map_pagetable(2*PDE_MAPPED_SIZE, pt2, pdflags);
    pd.map_pagetable(3*PDE_MAPPED_SIZE, pt3, pdflags);

    // Map in the kernel. We know constructing the frame variable is safe because we only map
    // the kernel in once.
    let ptflags = PTE_SUPERVISOR | PTE_WRITABLE | PTE_GLOBAL;
    let kernel_start = linker_sym!(__kernel_start);
    let kernel_end = linker_sym!(__kernel_end);
    for page in (kernel_start..kernel_end).step_by(PAGE_SIZE) {
        assert!(pd.has_pagetable(page));
        let frame  = unsafe { Frame::from_addr(page) };
        pd.map_page(page, frame, ptflags);
    }

    // Map in video memory. We know constructing the vmem_frame variable is safe because we only map
    // in video memory once.
    let vmem: usize = 0xB8000;
    let vmem_frame = unsafe { Frame::from_addr(vmem) };
    pd.map_page(vmem, vmem_frame, ptflags);

    // Mark code/rodata as readonly to prevent a few bugs.
    let ro_start = linker_sym!(__ro_start);
    let ro_end = linker_sym!(__ro_end);
    for page in (ro_start..ro_end).step_by(PAGE_SIZE) {
        pd.remove_pte_flags(page, PTE_WRITABLE);
    }

    // Set the global default kernel page directory. We're subverting the 
    KPD.init(pd);
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

