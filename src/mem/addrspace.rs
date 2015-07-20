use core::prelude::*;
use core::ops::{Index, IndexMut};
use core::slice;
use sync::rwlock::{RWLock, ReaderGuard, WriterGuard};
use util::KernResult;
use ::virt::{PageDirectory, PageTable, PageTableEntry, PDE_WRITABLE};
use ::phys::{FrameReserve, Frame};

/// An abstract representation of the address space. This enables the kernel to lock portions of
/// the address space for manipulation. Currently this is done using a RWLock but this should be
/// made more granular to track individual allocations.
pub struct AddressSpace {
    state: RWLock<AddressSpaceState>,
}

pub struct AddressSpaceState {
    resv: FrameReserve,
    pd: Frame<PageDirectory>,
}

/// Enables the user to read from the range [lo, hi) and guarantees these pages will not be written
/// to or unmapped.
pub struct AddressReader<'a> {
    guard: ReaderGuard<'a, AddressSpaceState>,
    slice: &'static [u8],
    lo: usize,
    hi: usize,
}

/// Enables the user full control of the range [lo, hi) allowing them to read, write, map, and
/// unmap pages.
pub struct AddressWriter<'a> {
    guard: WriterGuard<'a, AddressSpaceState>,
    slice: &'static mut [u8],
    lo: usize,
    hi: usize,
}

impl AddressSpace {
    
    pub fn new(pd: Frame<PageDirectory>) -> AddressSpace {
        AddressSpace {
            state: RWLock::new(AddressSpaceState {
                resv: FrameReserve::new(),
                pd: pd
            })
        }
    }

    pub fn lock_range_reader(&self, lo: usize, hi: usize) -> AddressReader {
        AddressReader {
            guard: self.state.lock_reader(),
            slice: unsafe { slice::from_raw_parts(lo as *const u8, hi - lo) },
            lo: lo,
            hi: hi,
        }
    }

    pub fn lock_range_writer(&self, lo: usize, hi: usize) -> AddressWriter {
        AddressWriter {
            guard: self.state.lock_writer(),
            slice: unsafe { slice::from_raw_parts_mut(lo as *mut u8, hi - lo) },
            lo: lo,
            hi: hi,
        }
    }

}

impl<'a> AddressReader<'a> {
    pub fn is_mapped(&self, addr: usize) -> bool {
        assert!(addr >= self.lo && addr < self.hi);
        self.guard.pd.has_page(addr)
    }
}

impl<'a> AddressWriter<'a> {
    
    pub fn is_mapped(&self, addr: usize) -> bool {
        assert!(addr >= self.lo && addr < self.hi);
        self.guard.pd.has_page(addr)
    }

    pub fn map_addr_reserved(&mut self, addr: usize, flags: PageTableEntry) {
        if !self.guard.pd.has_pagetable(addr) {
            let pt = PageTable::new(self.guard.resv.get_frame());
            self.guard.pd.map_pagetable(addr, pt, PDE_WRITABLE);
        }
        let frame = self.guard.resv.get_frame();
        self.guard.pd.map_page(addr, frame, flags);
    }

    pub fn map_addr_unreserved(&mut self, addr: usize, flags: PageTableEntry) -> KernResult<()> {
        if !self.guard.pd.has_pagetable(addr) {
            let pt = PageTable::new(try!(self.guard.resv.get_frame_unreserved()));
            self.guard.pd.map_pagetable(addr, pt, PDE_WRITABLE);
        }
        let frame = try!(self.guard.resv.get_frame_unreserved());
        self.guard.pd.map_page(addr, frame, flags);
        Ok(())
    }

}

impl<'a> Index<usize> for AddressReader<'a> {
    type Output = u8;
    fn index(&self, addr: usize) -> &u8 {
        assert!(self.guard.pd.has_page(addr));
        assert!(addr >= self.lo && addr < self.hi);
        &self.slice[addr - self.lo]
    }
}

impl<'a> Index<usize> for AddressWriter<'a> {
    type Output = u8;
    fn index(&self, addr: usize) -> &u8 {
        assert!(self.guard.pd.has_page(addr));
        assert!(addr >= self.lo && addr < self.hi);
        &self.slice[addr - self.lo]
    }
}

impl<'a> IndexMut<usize> for AddressWriter<'a> {
    fn index_mut(&mut self, addr: usize) -> &mut u8 {
        assert!(self.guard.pd.has_page(addr));
        assert!(addr >= self.lo && addr < self.hi);
        &mut self.slice[addr - self.lo]
    }
}

