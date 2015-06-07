//!
//! An interface to the List-based Memory Manager. This is not a particularly GOOD allocator but it
//! is specialized at working in a kernel environment where we can't just invoke sbrk to get more
//! heap space. 
//!
//! More information can be found here: http://www.cs.utah.edu/flux/oskit/html/oskit-wwwch25.html
//!
use core::prelude::*;
use core::intrinsics::volatile_copy_memory;
use util::align_bits;
use Allocator;
logger_init!(Trace);

const ALLOC_FLAGS: u32 = 0;

#[allow(dead_code,improper_ctypes)]
extern {
    fn lmm_init(lmm: &mut LMM);
    fn lmm_add_region(lmm: &mut LMM, region: &mut LMMRegion, addr: usize, size: usize);
    fn lmm_add_free(lmm: &mut LMM, addr: usize, size: usize);
    fn lmm_remove_free(lmm: &mut LMM, addr: usize, size: usize);
    fn lmm_alloc(lmm: &mut LMM, size: usize, flags: u32) -> usize;
    fn lmm_alloc_aligned(lmm: &mut LMM, size: usize, flags: u32, align_bits: u32, align_ofs: usize) -> usize;
    fn lmm_alloc_page(lmm: &mut LMM, flags: u32) -> usize;
    fn lmm_alloc_gen(lmm: &mut LMM, size: usize, flags: u32, align_bits: u32, align_ofs: usize, bounds_min: usize, bounds_max: usize) -> usize;
    fn lmm_avail(lmm: &mut LMM, flags: u32);
    fn lmm_find_free(lmm: &mut LMM, inout_addr: &mut usize, out_size: &mut usize, out_flags: &mut u32);
    fn lmm_free(lmm: &mut LMM, addr: usize, size: usize);
    fn lmm_free_page(lmm: &mut LMM, addr: usize);
}

#[repr(C)]
struct LMMNode {
    next: *mut LMMNode,
    size: usize,
}

#[repr(C)]
struct LMMRegion {
    next: *mut LMMRegion,
    nodes: *mut LMMNode,
    min: usize,
    max: usize,
    flags: u32,
    pri: u32,
    free: usize,
}

#[repr(C)]
struct LMM {
    regions: *mut LMMRegion,
}

pub struct LMMAllocator {
    lmm: LMM,
    region: LMMRegion,
}


pub const LMM_ALLOCATOR_INIT: LMMAllocator = LMMAllocator {
    lmm: LMM { regions: 0 as *mut LMMRegion },
    region: LMMRegion { 
        next: 0 as *mut LMMRegion,
        nodes: 0 as *mut LMMNode,
        min: 0,
        max: 0,
        flags: 0,
        pri: 0,
        free: 0
    }
};

impl LMMAllocator {

    pub fn init(&mut self, heap_start: usize, heap_end: usize) {
        trace!("initializing allocator with heap 0x{:x}-0x{:x} ({} bytes)", heap_start, heap_end, heap_end - heap_start);
        assert!(heap_start < heap_end);
        unsafe {
            lmm_init(&mut self.lmm);
            lmm_add_region(&mut self.lmm, &mut self.region, heap_start, heap_end - heap_start);
            lmm_add_free(&mut self.lmm, heap_start, heap_end - heap_start);
        }
    }

}

/// An address for 0-sized allocations.
static EMPTY: () = ();

impl Allocator for LMMAllocator {

    fn allocate_raw(&mut self, size: usize, align: usize) -> Option<usize> {
        trace!("trying to allocate {} bytes aligned to {:x}", size, align);
        
        if size == 0 {
            // If the allocation is 0 bytes, LMM can't handle it so we just give it a dummy
            // address.
            trace!("allocated 0 bytes at {:p}", &EMPTY);
            return Some(&EMPTY as *const () as usize);
        } else {
            // Otherwise we go to LMM for the allocation.
            let align_bits = align_bits(align) as u32;
            let align_ofs = 0;
            match unsafe { lmm_alloc_aligned(&mut self.lmm, size, ALLOC_FLAGS, align_bits, align_ofs) } {
                0 => {
                    trace!("could not allocate {} bytes", size);
                    None
                }
                x => {
                    trace!("allocated {} bytes at 0x{:x}", size, x);
                    Some(x)
                }
            }
        }
    }

    fn reallocate_raw(&mut self, old_addr: usize, old_size: usize, new_size: usize, align: usize) -> Result<usize, usize> {
        // LMM does not actually contain a realloc function so we are limited to just trying a new
        // allocation, copying bytes, and freeing the old one.
        self.allocate_raw(new_size, align).map(|new_addr| {
            // The allocation succeeded, copy bytes and free the old one.
            let src = old_addr as *mut u8;
            let dst = new_addr as *mut u8;
            unsafe { volatile_copy_memory(dst, src, old_size) };
            self.deallocate_raw(old_addr, old_size);
            new_addr
        }).ok_or(old_addr)
    }

    fn deallocate_raw(&mut self, addr: usize, size: usize) {
        trace!("freeing {} bytes at 0x{:x}", size, addr);
        
        // Only free if this is not a 0 byte allocation. If it is 0 bytes, it better be a pointer
        // to EMPTY or else we didn't allocate it!
        if size != 0 {
            unsafe { lmm_free(&mut self.lmm, addr, size) }
        } else {
            assert!(addr == &EMPTY as *const () as usize);
        }
    } 

}
