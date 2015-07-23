use core::prelude::*;
use core::{mem, fmt};
use phys::Frame;
use util::{asm, is_page_aligned, PAGE_SIZE};
use super::{addr_to_pde, addr_to_pte};
use super::pt::PageTable;

bitflags! {
    flags PageDirectoryEntry: u32 {
        const PDE_NONE         = 0x00000000,
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
        assert!(is_aligned!(frame, PAGE_SIZE*PAGE_SIZE));
        self.bits |= frame as u32;
    }

    /// Sets the page directory entry to point to the page table `pt`.
    ///
    /// # Panics
    ///
    /// If the page directory entry has its 4MB region flag set, this function panics.
    pub fn set_pagetable(&mut self, pt: Frame<PageTable>) {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr = unsafe { pt.into_addr() as usize };
        assert!(is_page_aligned(pt_addr));
        self.bits |= pt_addr as u32;
    }

    /// Removes a page table from the page directory.
    ///
    /// # Panics
    ///
    /// This function panics if this entry does not have a mapped page table or its 4MB region flag
    /// is set.
    pub fn remove_pagetable(&mut self) -> Frame<PageTable> {
        assert!(!self.contains(PDE_4MBREGION));
        let pt_addr: usize = (self.bits & PDE_FRAMEMASK.bits) as usize;
        assert!(pt_addr != 0);
        self.clear();

        // We know it's safe to construct this frame because we know we put a page table INTO the
        // page directory.
        unsafe { Frame::from_addr(pt_addr) }
    }
  
}

pub struct PageDirectory {
    pdes: [PageDirectoryEntry; 1024] 
}

impl PageDirectory {
    
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
    pub fn map_pagetable(&mut self, addr: usize, pt: Frame<PageTable>, flags: PageDirectoryEntry) {
        assert!(!self.has_pagetable(addr));
        let pde = addr_to_pde(addr);
        self.pdes[pde].clear();
        self.pdes[pde].set_pagetable(pt);
        self.pdes[pde].insert(flags | PDE_PRESENT);
    }

    pub fn unmap_pagetable(&mut self, addr: usize) -> Frame<PageTable> {
        assert!(self.has_pagetable(addr));
        let pde = addr_to_pde(addr);
        self.pdes[pde].remove_pagetable()
    }

    /// Returns whether a specified address has a page table or not.
    pub fn has_pagetable(&self, addr: usize) -> bool {
        let pde = addr_to_pde(addr);
        self.pdes[pde].contains(PDE_PRESENT)
    }

}

impl fmt::Debug for PageDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageDirectory@{:x}", self as *const PageDirectory as usize)
    }
}

