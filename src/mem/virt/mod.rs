use core::prelude::*;
use core::mem;
use util::{is_aligned, is_page_aligned, PAGE_SIZE};
use phys;
use phys::Frame;
use rawbox::RawBox;

bitflags! {
    flags PageTableEntry: u32 {
        const PTE_EMPTY        = 0x00000000,
        const PTE_PRESENT      = 0x00000001,
        const PTE_WRITABLE     = 0x00000002,
        const PTE_SUPERVISOR   = 0x00000004,
        const PTE_WRITETHROUGH = 0x00000008,
        const PTE_CACHEDISABLE = 0x00000010,
        const PTE_ACCESSED     = 0x00000020,
        const PTE_DIRTY        = 0x00000040,
        const PTE_GLOBAL       = 0x00000100,
        const PTE_FRAMEMASK    = 0xfffff000,
    }
}

impl PageTableEntry {

    pub fn set_frame(&mut self, frame: RawBox<Frame>) {
        self.clear_frame();
        self.bits |= frame.to_addr() as u32;
    }

    pub fn clear_frame(&mut self) {
        self.bits &= !PTE_FRAMEMASK.bits;
    }

    pub fn get_frame(&mut self) -> RawBox<Frame> {
        let frame_addr = (self.bits & PTE_FRAMEMASK.bits) as usize;
        Frame::from_addr(frame_addr)
    }
}

bitflags! {
    flags PageDirectoryEntry: u32 {
        const PDE_EMPTY        = 0x00000000,
        const PDE_PRESENT      = 0x00000001,
        const PDE_WRITABLE     = 0x00000002,
        const PDE_SUPERVISOR   = 0x00000004,
        const PDE_WRITETHROUGH = 0x00000008,
        const PDE_CACHEDISABLE = 0x00000010,
        const PDE_ACCESSED     = 0x00000020,
        const PDE_DIRTY        = 0x00000040, //*
        const PDE_4MBREGION    = 0x00000080,
        const PDE_GLOBAL       = 0x00000100, //*
        const PDE_FRAMEMASK    = 0xfffff000,
    }
    //* Indicates these flags are only valid for 4MB regions.
}

impl PageDirectoryEntry {
    
    pub fn get_pagetable(&mut self) -> RawBox<PageTable> {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = (self.bits & PDE_FRAMEMASK.bits) as usize;
        RawBox::from_raw(pt_addr as *mut PageTable)
    }

    pub fn set_pagetable(&mut self, pt: &PageTable) {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = pt as *const PageTable as usize;
        assert!(is_page_aligned(pt_addr));
        self.bits |= pt_addr as u32;

    }

    pub fn set_4mbframe(&mut self, frame: usize) {
        assert!(self.contains(PDE_4MBREGION));
        assert!(is_aligned(frame, PAGE_SIZE*PAGE_SIZE));
        self.bits |= frame as u32;
    }

}

pub struct PageDirectory {
    pdes: [PageDirectoryEntry; 1024] 
}

impl PageDirectory {
    
    pub fn new() -> Option<RawBox<PageDirectory>> {
        phys::get_frame().map(|f| {
            let mut pd: RawBox<PageDirectory> = f.allocate();
            pd.clear();
            pd
        })
    }

    pub fn clear(&mut self) {
        for pde in self.pdes.as_mut() {
            pde.clear()
        }
    }

}

pub struct PageTable {
    ptes: [PageTableEntry; 1024]
}

impl PageTable {

    pub fn new() -> Option<RawBox<PageTable>> {
        phys::get_frame().map(|f| {
             let mut pt: RawBox<PageTable> = f.allocate();
             pt.clear();
             pt
        })
    }

    pub fn clear(&mut self) {
        for pte in self.ptes.as_mut() {
            pte.clear()
        }
    }

}

pub fn init() {
    // In light of static_assert being removed, this will have to do.
    assert!(mem::size_of::<PageTable>() == PAGE_SIZE);
    assert!(mem::size_of::<PageDirectory>() == PAGE_SIZE);
}
