use core::prelude::*;
use core::mem;
use core::fmt;
use core::fmt::{Debug, Formatter};
use util::{is_aligned, is_page_aligned, PAGE_SIZE};
use phys;
use phys::Frame;
use rawbox::RawBox;

const ENTRY_MASK: usize = 0x3FF;
const PT_SHIFT: usize = 12;
const PD_SHIFT: usize = 22;
const PTE_ALIGN: usize = (1 << PT_SHIFT);
const PDE_ALIGN: usize = (1 << PD_SHIFT);
pub const PTE_MAPPED_SIZE: usize = (1 << PT_SHIFT);
pub const PDE_MAPPED_SIZE: usize = (1 << PD_SHIFT);
pub const PD_RECMAP_ADDR: usize = -PDE_MAPPED_SIZE;

// Converts an address to its page table index.
fn addr_to_pte (addr: usize) -> usize {
    (addr >> PT_SHIFT) & ENTRY_MASK
}

// Converts an address to its page directory index.
fn addr_to_pde (addr: usize) -> usize {
    (addr >> PD_SHIFT) & ENTRY_MASK
}

bitflags! {
    flags PageDirectoryEntry: u32 {
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
    
    pub fn set_4mbframe(&mut self, frame: usize) {
        assert!(self.contains(PDE_4MBREGION));
        assert!(is_aligned(frame, PAGE_SIZE*PAGE_SIZE));
        self.bits |= frame as u32;
    }

    pub fn set_pagetable(&mut self, pt: RawBox<PageTable>) {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = pt.to_raw() as usize;
        assert!(is_page_aligned(pt_addr));
        self.bits |= pt_addr as u32;
    }

    pub fn remove_pagetable(&mut self) -> RawBox<PageTable> {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = (self.bits & PDE_FRAMEMASK.bits) as usize;
        self.clear();
        RawBox::from_raw(pt_addr as *mut PageTable)
    }
   
    pub fn borrow_pagetable(&self) -> &PageTable {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = (self.bits & PDE_FRAMEMASK.bits) as usize;
        assert!(pt_addr != 0);
        unsafe { &*(pt_addr as *mut PageTable) }
    }
    
    pub fn borrow_pagetable_mut(&mut self) -> &mut PageTable {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = (self.bits & PDE_FRAMEMASK.bits) as usize;
        assert!(pt_addr != 0);
        unsafe { &mut*(pt_addr as *mut PageTable) }
    }

}

bitflags! {
    flags PageTableEntry: u32 {
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

    pub fn set_page(&mut self, frame: RawBox<Frame>) {
        assert!(!self.intersects(PTE_FRAMEMASK));
        self.bits |= frame.to_addr() as u32;
    }

    pub fn remove_page(&mut self) -> RawBox<Frame> {
        let frame_addr = (self.bits & PTE_FRAMEMASK.bits) as usize;
        self.clear();
        Frame::from_addr(frame_addr)
    }

    pub fn borrow_frame(&self) -> &Frame {
        let frame_addr = (self.bits & PTE_FRAMEMASK.bits) as usize;
        assert!(frame_addr != 0);
        unsafe { &*(frame_addr as *mut Frame) }
    }

    pub fn borrow_frame_mut(&mut self) -> &mut Frame {
        let frame_addr = (self.bits & PTE_FRAMEMASK.bits) as usize;
        assert!(frame_addr != 0);
        unsafe { &mut*(frame_addr as *mut Frame) }
    }

}

pub struct PageDirectory {
    pdes: [PageDirectoryEntry; 1024] 
}

impl PageDirectory {
    
    pub fn new() -> Option<RawBox<PageDirectory>> {
        phys::get_frame().map(|f| {
            let mut pd: RawBox<PageDirectory> = f.allocate();
            unsafe { pd.clear() };
            pd
        })
    }

    /// Removes all mappings and marks all entries as not present.
    ///
    /// # Safety
    ///
    /// This is unsafe because it may leak any page tables that are mapped in.
    pub unsafe fn clear(&mut self) {
        for pde in self.pdes.as_mut() {
            pde.clear()
        }
    }

    pub fn map_pagetable(&mut self, addr: usize, pt: RawBox<PageTable>, flags: PageDirectoryEntry) {
        assert!(!self.has_pagetable(addr));
        let pde = addr_to_pde(addr);
        self.pdes[pde].clear();
        self.pdes[pde].set_pagetable(pt);
        self.pdes[pde].insert(flags | PDE_PRESENT);
    }

    pub fn has_pagetable(&self, addr: usize) -> bool {
        let pde = addr_to_pde(addr);
        self.pdes[pde].contains(PDE_PRESENT)
    }

    pub fn map_page(&mut self, addr: usize, frame: RawBox<Frame>, flags: PageTableEntry) {
        assert!(self.has_pagetable(addr));
        assert!(!self.has_page(addr));
        let pde = addr_to_pde(addr);
        self.pdes[pde].borrow_pagetable_mut().map_page(addr, frame, flags)
    }

    pub fn has_page(&self, addr: usize) -> bool {
        let pde = addr_to_pde(addr);
        self.pdes[pde].borrow_pagetable().has_page(addr)
    }

    pub unsafe fn as_pagetable(&mut self) -> RawBox<PageTable> {
        RawBox::from_raw(self as *mut PageDirectory as *mut PageTable)
    }

    pub fn remove_pte_flags(&mut self, addr: usize, flags: PageTableEntry) {
        self.pdes[addr_to_pde(addr)].borrow_pagetable_mut().remove_flags(addr, flags);
    }

}

impl Debug for PageDirectory {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "PageDirectory@{:x}", self as *const PageDirectory as usize)
    }
}

pub struct PageTable {
    ptes: [PageTableEntry; 1024]
}

impl PageTable {

    pub fn new() -> Option<RawBox<PageTable>> {
        phys::get_frame().map(|f| {
             let mut pt: RawBox<PageTable> = f.allocate();
             unsafe { pt.clear() };
             pt
        })
    }

    pub unsafe fn clear(&mut self) {
        for pte in self.ptes.as_mut() {
            pte.clear()
        }
    }

    pub fn map_page(&mut self, addr: usize, frame: RawBox<Frame>, flags: PageTableEntry) {
        assert!(!self.has_page(addr));
        let pte = addr_to_pte(addr);
        self.ptes[pte].clear();
        self.ptes[pte].set_page(frame);
        self.ptes[pte].insert(flags | PTE_PRESENT);
    }

    pub fn has_page(&self, addr: usize) -> bool {
        // Here we are assuming this is the RIGHT page table since we can't 
        // check that the upper bits of the address correspond to this page table.
        self.ptes[addr_to_pte(addr)].contains(PTE_PRESENT)
    }

    pub fn remove_flags(&mut self, addr: usize, flags: PageTableEntry) {
        assert!(self.has_page(addr));
        assert!(!flags.intersects(PTE_PRESENT | PTE_FRAMEMASK));
        self.ptes[addr_to_pte(addr)].remove(flags);
    }

    pub fn add_flags(&mut self, addr: usize, flags: PageTableEntry) {
        assert!(self.has_page(addr));
        assert!(!flags.intersects(PTE_PRESENT | PTE_FRAMEMASK));
        self.ptes[addr_to_pte(addr)].insert(flags);
    }

}

impl Debug for PageTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "PageTable@{:x}", self as *const PageTable as usize)
    }
}

pub fn init() {
    // In light of static_assert being removed, this will have to do.
    assert!(mem::size_of::<PageTable>() == PAGE_SIZE);
    assert!(mem::size_of::<PageDirectory>() == PAGE_SIZE);
}
