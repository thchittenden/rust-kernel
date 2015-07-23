use core::prelude::*;
use core::{mem, fmt};
use phys::Frame;
use util::{is_page_aligned, PAGE_SIZE};
use super::{addr_to_pde, addr_to_pte};

bitflags! {
    flags PageTableEntry: u32 {
        const PTE_NONE         = 0x00000000,
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
    pub fn set_frame(&mut self, frame: Frame<()>) {
        assert!(!self.intersects(PTE_FRAMEMASK));
        let addr = unsafe { frame.into_addr() };
        self.bits |= addr as u32;
    }

    /// Removes the page from the page table entry. TODO INVLPG?
    ///
    /// # Panics
    ///
    /// This funcion panics if this entry does not have a mapped frame.
    pub fn remove_frame(&mut self) -> Frame<()> {
        let frame_addr = (self.bits & PTE_FRAMEMASK.bits) as usize;
        assert!(frame_addr != 0);
        self.clear();

        // We know it's safe to construct this owned pointer because if we know we put a unique
        // pointer INTO the page table.
        unsafe { Frame::from_addr(frame_addr) }
    }
}

pub struct PageTable {
    ptes: [PageTableEntry; 1024]
}

impl PageTable {

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
    pub fn map_page(&mut self, addr: usize, frame: Frame<()>, flags: PageTableEntry) {
        assert!(!self.has_page(addr));
        let pte = addr_to_pte(addr);
        self.ptes[pte].clear();
        self.ptes[pte].set_frame(frame);
        self.ptes[pte].insert(flags | PTE_PRESENT);
    }

    pub fn unmap_page(&mut self, addr: usize) -> Frame<()> {
        assert!(self.has_page(addr));
        let pte = addr_to_pte(addr);
        self.ptes[pte].remove_frame()
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

impl fmt::Debug for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageTable@{:x}", self as *const PageTable as usize)
    }
}

