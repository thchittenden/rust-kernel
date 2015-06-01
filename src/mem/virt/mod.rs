use core::prelude::*;
use core::mem;
use core::fmt;
use core::fmt::{Debug, Formatter};
use phys;
use phys::Frame;
use util::{is_aligned, is_page_aligned, PAGE_SIZE};
use util::rawbox::{RawBox, Unallocated};

const ENTRY_MASK: usize = 0x3FF;
const PT_SHIFT: usize = 12;
const PD_SHIFT: usize = 22;
pub const PTE_MAPPED_SIZE: usize = (1 << PT_SHIFT);
pub const PDE_MAPPED_SIZE: usize = (1 << PD_SHIFT);

#[allow(unsigned_negation)]
// This is intentional. We want it to be the last page directory entry.
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
    
    /// Sets the page directory entry to point to the 4MB region pointed to by `frame`.
    ///
    /// # Panics
    ///
    /// If this page directory entry does not have its 4MB region flag set, this function panics.
    pub fn set_4mbframe(&mut self, frame: usize) {
        assert!(self.contains(PDE_4MBREGION));
        assert!(is_aligned(frame, PAGE_SIZE*PAGE_SIZE));
        self.bits |= frame as u32;
    }

    /// Sets the page directory entry to point to the page table `pt`.
    ///
    /// # Panics
    ///
    /// If the page directory entry has its 4MB region flag set, this function panics.
    pub fn set_pagetable(&mut self, pt: RawBox<PageTable>) {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = pt.into_raw() as usize;
        assert!(is_page_aligned(pt_addr));
        self.bits |= pt_addr as u32;
    }

    /// Removes a page table from the page directory.
    ///
    /// # Panics
    ///
    /// This function panics if this entry does not have a mapped page table or its 4MB region flag
    /// is set.
    pub fn remove_pagetable(&mut self) -> RawBox<PageTable> {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = (self.bits & PDE_FRAMEMASK.bits) as usize;
        assert!(pt_addr != 0);
        self.clear();

        // We know it's safe to construct this owned pointer because if we know we put a unique
        // pointer INTO the page directory.
        unsafe { RawBox::from_raw(pt_addr as *mut PageTable) }
    }
  
    /// Borrows a page table from the page directory entry.  
    ///
    /// We know this is safe because the entry owns the page table pointer.
    ///
    /// # Panics
    ///
    /// This function panics if this entry does not have a mapped page table or its 4MB region flag
    /// is set.
    pub fn borrow_pagetable(&self) -> &PageTable {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = (self.bits & PDE_FRAMEMASK.bits) as usize;
        assert!(pt_addr != 0);
        unsafe { &*(pt_addr as *mut PageTable) }
    }
    
    /// Mutably borrows a page table from the page table from the page directory entry.
    ///
    /// # Panics
    ///
    /// This function panics if this entry does not have a mapped page table or its 4MB region flag
    /// is set.
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

    /// Sets the frame address of the entry to the given owned frame. 
    ///
    /// # Panics
    ///
    /// This function panics if this entry already has a mapped frame.
    pub fn set_page(&mut self, frame: RawBox<Frame>) {
        assert!(!self.intersects(PTE_FRAMEMASK));
        self.bits |= frame.into_raw() as u32;
    }

    /// Removes the page from the page table entry. TODO INVLPG?
    ///
    /// # Panics
    ///
    /// This funcion panics if this entry does not have a mapped frame.
    pub fn remove_page(&mut self) -> RawBox<Frame> {
        let frame_addr = (self.bits & PTE_FRAMEMASK.bits) as usize;
        assert!(frame_addr != 0);
        self.clear();

        // We know it's safe to construct this owned pointer because if we know we put a unique
        // pointer INTO the page table.
        unsafe { Frame::from_addr(frame_addr) }
    }

    /// Borrows the frame from the page table entry. 
    ///
    /// This is safe because the entry owns the frame pointer.
    ///
    /// # Panics
    ///
    /// This function panics if this entry does not have a mapped frame.
    pub fn borrow_frame(&self) -> &Frame {
        let frame_addr = (self.bits & PTE_FRAMEMASK.bits) as usize;
        assert!(frame_addr != 0);
        unsafe { &*(frame_addr as *mut Frame) }
    }

    /// Mutably borrows the frame from the page table entry.
    ///
    /// # Panics
    ///
    /// This function panics if this entry does not have a mapped frame.
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
    
    /// Tries to allocate a new, cleared page directory from the free frame list.
    pub fn new() -> Option<RawBox<PageDirectory>> {
        phys::get_frame().map(|f| {
            let mut pd: RawBox<PageDirectory> = f.allocate();
            unsafe { pd.clear() }; // This is safe because there is nothing mapped in.
            pd
        })
    }

    /// Removes all mappings and marks all entries as not present.
    ///
    /// # Safety
    ///
    /// This is unsafe because it will leak any page tables that are currently mapped.
    pub unsafe fn clear(&mut self) {
        for pde in self.pdes.as_mut() {
            pde.clear()
        }
    }

    /// Maps a page table for the specified address with the given flags.
    ///
    /// # Panics
    ///
    /// This function panics if th address already has a page table mapped.
    pub fn map_pagetable(&mut self, addr: usize, pt: RawBox<PageTable>, flags: PageDirectoryEntry) {
        assert!(!self.has_pagetable(addr));
        let pde = addr_to_pde(addr);
        self.pdes[pde].clear();
        self.pdes[pde].set_pagetable(pt);
        self.pdes[pde].insert(flags | PDE_PRESENT);
    }

    /// Returns whether a specified address has a page table or not.
    pub fn has_pagetable(&self, addr: usize) -> bool {
        let pde = addr_to_pde(addr);
        self.pdes[pde].contains(PDE_PRESENT)
    }

    /// Maps a frame for the specified address with the given flag.
    ///
    /// # Panics
    ///
    /// This function panics if this page does not have an associated page table.
    pub fn map_page(&mut self, addr: usize, frame: RawBox<Frame>, flags: PageTableEntry) {
        assert!(self.has_pagetable(addr));
        assert!(!self.has_page(addr));
        let pde = addr_to_pde(addr);
        self.pdes[pde].borrow_pagetable_mut().map_page(addr, frame, flags)
    }

    /// Returns whether a specified address has a mapped frame or not.
    pub fn has_page(&self, addr: usize) -> bool {
        let pde = addr_to_pde(addr);
        self.pdes[pde].borrow_pagetable().has_page(addr)
    }

    /// Converts this page directory into a page table. 
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller needs to ensure they use properly map this page
    /// table. TODO it may be better to not expose this functionality and instead just provide a
    /// `map_recursive` function.
    pub unsafe fn as_pagetable(&mut self) -> RawBox<PageTable> {
        RawBox::from_raw(self as *mut PageDirectory as *mut PageTable)
    }

    /// Removes flags from the page table for the given address.
    /// 
    /// # Panics
    ///
    /// This function panics if there is no page table for the given address or the flags to be
    /// removed would result in unsafe behavior (removing the present or frame mask flags).
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

    /// Tries to allocate a new, cleared page table from the free frame list.
    pub fn new() -> Option<RawBox<PageTable>> {
        phys::get_frame().map(|f| {
             let mut pt: RawBox<PageTable> = f.allocate();
             unsafe { pt.clear() }; // This is safe because there is nothing mapped in.
             pt
        })
    }

    /// Clears all entries in the page table.
    ///
    /// # Safety
    ///
    /// This is unsafe because it will leak any frames that are mapped in.
    pub unsafe fn clear(&mut self) {
        for pte in self.ptes.as_mut() {
            pte.clear()
        }
    }

    /// Maps a frame to the given address. This assumes this is the correct page table for the
    /// given address since there's no way to verify that the upper bits of the address correspond
    /// to this page table.
    ///
    /// # Panics
    ///
    /// This function will panic if there is already a frame mapped to the given address.
    pub fn map_page(&mut self, addr: usize, frame: RawBox<Frame>, flags: PageTableEntry) {
        assert!(!self.has_page(addr));
        let pte = addr_to_pte(addr);
        self.ptes[pte].clear();
        self.ptes[pte].set_page(frame);
        self.ptes[pte].insert(flags | PTE_PRESENT);
    }

    /// Returns whether an address has a mapped frame.
    pub fn has_page(&self, addr: usize) -> bool {
        // Here we are assuming this is the RIGHT page table since we can't 
        // check that the upper bits of the address correspond to this page table.
        self.ptes[addr_to_pte(addr)].contains(PTE_PRESENT)
    }

    /// Removes a set of flags from the page table entry for a given address.
    ///
    /// # Panics
    ///
    /// This function will panic if it would remove flags that would result in unsafe behaviour
    /// such as the present or frame mask flags.
    ///
    /// This function will panic if there is no mapped frame for this address.
    pub fn remove_flags(&mut self, addr: usize, flags: PageTableEntry) {
        assert!(self.has_page(addr));
        assert!(!flags.intersects(PTE_PRESENT | PTE_FRAMEMASK));
        self.ptes[addr_to_pte(addr)].remove(flags);
    }

    /// Adds a set of flags from the page table entry for a given address.
    ///
    /// # Panics
    ///
    /// This function will panic if it would add flags that would result in unsafe behaviour
    /// such as the present or frame mask flags.
    ///
    /// This function will panic if there is no mapped frame for this address.
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

/// Initializes the virtual memory module. This merely asserts that page tables and page
/// directories were compiled to the correct size for x86 systems.
pub fn init() {
    // In light of static_assert being removed, this will have to do.
    assert!(mem::size_of::<PageTable>() == PAGE_SIZE);
    assert!(mem::size_of::<PageDirectory>() == PAGE_SIZE);
}
