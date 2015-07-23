pub mod pd;
pub mod pt;
pub mod page;
pub mod constants;

use core::prelude::*;
use core::ops::{Index, IndexMut, Range};
use core::slice;
use sync::rwlock::{RWLock, ReaderGuard, WriterGuard};
use util::{asm, page_align, page_align_up, PAGE_SIZE, KernResult};
use self::pd::{PageDirectory, PageDirectoryEntry, PDE_SUPERVISOR, PDE_WRITABLE};
use self::pt::{PageTable, PageTableEntry, PTE_SUPERVISOR, PTE_WRITABLE, PTE_GLOBAL};
use self::constants::*;
use self::page::Page;
use ::phys::{FrameReserve, Frame};
logger_init!(Trace);

// Converts an address to its page table index.
fn addr_to_pte (addr: usize) -> usize {
    (addr >> PT_SHIFT) & ENTRY_OFF_MASK
}

// Converts an address to its page directory index.
fn addr_to_pde (addr: usize) -> usize {
    (addr >> PD_SHIFT) & ENTRY_OFF_MASK
}

struct AddressSpaceState {
    resv: FrameReserve,
}

/// An abstract representation of the address space. This enables the kernel to lock portions of
/// the address space for manipulation. Currently this is done using a RWLock but this should be
/// made more granular to track individual allocations.
pub struct AddressSpace {
    state: RWLock<AddressSpaceState>,
    cr3: usize,
}

impl AddressSpaceState {

    /// Gets the page directory.
    fn get_pd(&self) -> Page<PageDirectory> {
        unsafe { Page::from_addr(PD_ADDR) }
    }

    /// Gets the page table for the given page directory entry.
    fn get_pt(&self, pde: usize) -> Page<PageTable> {
        let addr = PT_BASE_ADDR + pde * PTE_MAP_SIZE;
        unsafe { Page::from_addr(addr) }
    }

    /// Maps a pagetable in from the reserved pool and clears it.
    fn map_pagetable_reserved(&mut self, addr: usize, flags: PageDirectoryEntry) {
        // We know it's safe to construct this page table because we clear it immediately after.
        // We know it's safe to clear the page table because there shouldn't be anything in it.
        unsafe { 
            let frame = self.resv.get_frame().cast::<PageTable>();
            self.get_pd().map_pagetable(addr, frame, flags);
            self.get_pt(addr_to_pde(addr)).clear();
        }
    }

    /// Tries to map a pagetable in from the unreserved pool and clears it.
    fn map_pagetable_unreserved(&mut self, addr: usize, flags: PageDirectoryEntry) -> KernResult<()> {
        // We know it's safe to construct this page table because we clear it immediately after.
        // We know it's safe to clear the page table because there shouldn't be anything in it.
        unsafe { 
            let frame = try!(self.resv.get_frame_unreserved()).cast::<PageTable>();
            self.get_pd().map_pagetable(addr, frame, flags);
            self.get_pt(addr_to_pde(addr)).clear();
        }
        Ok(())
    }

    /// Returns whether an address has a corresponding page table or not.
    fn has_pagetable(&self, addr: usize) -> bool {
        self.get_pd().has_pagetable(addr)
    }

    /// Maps a pre-existing page.
    fn map_page(&mut self, addr: usize, frame: Frame<()>, flags: PageTableEntry) {
        assert!(self.get_pd().has_pagetable(addr));
        self.get_pt(addr_to_pde(addr)).map_page(addr, frame, flags);
    }

    /// Maps a page in from the reserved pool.
    fn map_page_reserved(&mut self, addr: usize, flags: PageTableEntry) {
        assert!(self.get_pd().has_pagetable(addr));
        let frame = self.resv.get_frame();
        self.get_pt(addr_to_pde(addr)).map_page(addr, frame, flags);
    }

    /// Tries to map a page in from the unreserved pool.
    fn map_page_unreserved(&mut self, addr: usize, flags: PageTableEntry) -> KernResult<()> {
        assert!(self.get_pd().has_pagetable(addr));
        let frame = try!(self.resv.get_frame_unreserved());
        self.get_pt(addr_to_pde(addr)).map_page(addr, frame, flags);
        Ok(())
    }

    /// Unmaps a page.
    fn unmap_page(&mut self, addr: usize) -> Frame<()> {
        assert!(self.has_page(addr));
        self.get_pt(addr_to_pde(addr)).unmap_page(addr)
    }
    
    /// Returns whether an address has a corresponding page or not.
    fn has_page(&self, addr: usize) -> bool {
        self.get_pd().has_pagetable(addr) && self.get_pt(addr_to_pde(addr)).has_page(addr)
    }

}

impl AddressSpace {
    
    /// Locks a memory range for reading. 
    ///
    /// # Panics
    ///
    /// This function panics if the address space is not the currently active address space. This
    /// would indicate that the index operations would not be validated and could page fault. 
    pub fn lock_range_reader(&self, lo: usize, hi: usize) -> AddressReader {
        let lo = page_align(lo);
        let hi = page_align_up(hi);
        let lock = self.state.lock_reader();
        AddressReader {
            guard: lock,
            slice: unsafe { slice::from_raw_parts(lo as *const u8, hi - lo) },
            lo: lo,
            hi: hi,
        }
    }

    pub fn lock_page_reader(&self, addr: usize) -> AddressReader {
        let addr = page_align(addr);
        let lock = self.state.lock_reader();
        AddressReader {
            guard: lock,
            slice: unsafe { slice::from_raw_parts(addr as *const u8, PAGE_SIZE) },
            lo: addr,
            hi: addr + PAGE_SIZE,
        }
    }

    /// Locks a memory range for writing.
    ///
    /// # Panics
    ///
    /// This function panics if the address space is not the currently active address space. This
    /// would indicate that the index operations would not be validated and could page fault. 
    pub fn lock_range_writer(&self, lo: usize, hi: usize) -> AddressWriter {
        let lo = page_align(lo);
        let hi = page_align_up(hi);
        let lock = self.state.lock_writer();
        AddressWriter {
            guard: lock,
            slice: unsafe { slice::from_raw_parts_mut(lo as *mut u8, hi - lo) },
            lo: lo,
            hi: hi,
        }
    }

    pub fn lock_page_writer(&self, addr: usize) -> AddressWriter {
        let addr = page_align(addr);
        let lock = self.state.lock_writer();
        AddressWriter {
            guard: lock,
            slice: unsafe { slice::from_raw_parts_mut(addr as *mut u8, PAGE_SIZE) },
            lo: addr,
            hi: addr + PAGE_SIZE,
        }
    }

    /// Makes this the 
    pub fn activate(&self) {
        asm::set_cr3(self.cr3);
    }

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

impl<'a> AddressReader<'a> {
    pub fn has_page(&self, addr: usize) -> bool {
        assert!(addr >= self.lo && addr < self.hi);
        self.guard.get_pd().has_pagetable(addr)
            && self.guard.get_pt(addr_to_pde(addr)).has_page(addr)
    }
}

impl<'a> AddressWriter<'a> {
    
    pub fn has_page(&self, addr: usize) -> bool {
        assert!(addr >= self.lo && addr < self.hi);
        self.guard.has_page(addr)
    }

    pub fn map_page(&mut self, addr: usize, frame: Frame<()>, flags: PageTableEntry) {
        assert!(addr >= self.lo && addr < self.hi);
        assert!(self.guard.has_pagetable(addr));
        self.guard.map_page(addr, frame, flags);
    }

    pub fn map_page_reserved(&mut self, addr: usize, flags: PageTableEntry) {
        assert!(addr >= self.lo && addr < self.hi);
        if !self.guard.has_pagetable(addr) {
            self.guard.map_pagetable_reserved(addr, PDE_WRITABLE);
        }
        self.guard.map_page_reserved(addr, flags);
    }

    pub fn map_page_unreserved(&mut self, addr: usize, flags: PageTableEntry) -> KernResult<()> {
        assert!(addr >= self.lo && addr < self.hi);
        if !self.guard.has_pagetable(addr) {
            try!(self.guard.map_pagetable_unreserved(addr, PDE_WRITABLE));
        }
        try!(self.guard.map_page_unreserved(addr, flags));
        Ok(())
    }

    pub fn map_all_unreserved(&mut self, flags: PageTableEntry) -> KernResult<()> {
        for page in (self.lo..self.hi).step_by(PAGE_SIZE) {
            if !self.has_page(page) {
                try!(self.map_page_unreserved(page, flags));
            }
        }
        Ok(())
    }

    pub fn unmap_page(&mut self, addr: usize) -> Frame<()> {
        assert!(addr >= self.lo && addr < self.hi);
        self.guard.unmap_page(addr)
    }   

}

impl<'a> Index<usize> for AddressReader<'a> {
    type Output = u8;
    fn index(&self, addr: usize) -> &u8 {
        assert!(self.guard.has_page(addr));
        assert!(addr >= self.lo && addr < self.hi);
        &self.slice[addr - self.lo]
    }
}

impl<'a> Index<usize> for AddressWriter<'a> {
    type Output = u8;
    fn index(&self, addr: usize) -> &u8 {
        assert!(self.guard.has_page(addr));
        assert!(addr >= self.lo && addr < self.hi);
        &self.slice[addr - self.lo]
    }
}

impl<'a> Index<Range<usize>> for AddressWriter<'a> {
    type Output = [u8];
    fn index(&self, range: Range<usize>) -> &[u8] {
        assert!(range.start >= self.lo && range.start < self.hi);
        assert!(range.end > self.lo && range.end <= self.hi);
        &self.slice[range.start-self.lo..range.end-self.lo]
    }
}

impl<'a> IndexMut<usize> for AddressWriter<'a> {
    fn index_mut(&mut self, addr: usize) -> &mut u8 {
        assert!(self.guard.has_page(addr));
        assert!(addr >= self.lo && addr < self.hi);
        &mut self.slice[addr - self.lo]
    }
}

impl<'a> IndexMut<Range<usize>> for AddressWriter<'a> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut [u8] {
        assert!(range.start >= self.lo && range.start < self.hi);
        assert!(range.end > self.lo && range.end <= self.hi);
        &mut self.slice[range.start-self.lo..range.end-self.lo]
    }
}

/// Initializes the virtual memory module, enables paging, and returns the initial AddressSpace
/// that direct maps the kernel.
pub fn init() -> AddressSpace {
    
    // When we enter this function, paging is off so it is safe to manipulate these frames
    // directly.
    let resv = FrameReserve::new();
    resv.reserve(5).expect("unable to allocate initial kernel frames");
    
    // Get the frames. We know clearing these is safe because they are brand new.
    let mut pd = resv.get_frame().emplace(|pd: &mut PageDirectory| unsafe { pd.clear() });
    let mut pt0 = resv.get_frame().emplace(|pt: &mut PageTable| unsafe { pt.clear() });
    let mut pt1 = resv.get_frame().emplace(|pt: &mut PageTable| unsafe { pt.clear() });
    let mut pt2 = resv.get_frame().emplace(|pt: &mut PageTable| unsafe { pt.clear() });
    let mut pt3 = resv.get_frame().emplace(|pt: &mut PageTable| unsafe { pt.clear() });
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
    let pdpt = unsafe { Frame::<()>::from_addr(pd.get_addr()).allocate_raw::<PageTable>() };
    pd.map_pagetable(PT_BASE_ADDR, pdpt, pdflags);
    
    {   // Perform all mapping operations here.
        let mut map_page = |addr, frame, flags| {
            if addr < 1*PDE_MAP_SIZE {
                pt0.map_page(addr, frame, flags);
            } else if addr < 2*PDE_MAP_SIZE {
                pt1.map_page(addr, frame, flags);
            } else if addr < 3*PDE_MAP_SIZE {
                pt2.map_page(addr, frame, flags);
            } else if addr < 4*PDE_MAP_SIZE {
                pt3.map_page(addr, frame, flags);
            } else {
                panic!("kernel exceeded page table allocation");
            }
        };
       
        // Map in the kernel. We know constructing the frame variable is safe because we only map
        // the kernel in once. 
        let ptflags = PTE_SUPERVISOR | PTE_WRITABLE | PTE_GLOBAL;
        let kernel_start = linker_sym!(__kernel_start);
        let kernel_end = linker_sym!(__kernel_end);
        for page in (kernel_start..kernel_end).step_by(PAGE_SIZE) {
            let frame = unsafe { Frame::from_addr(page) };
            map_page(page, frame, ptflags);
        }
        
        // Map in video memory. We know constructing the vmem_frame variable is safe because we only map
        // in video memory once.
        let vmem: usize = 0xB8000;
        let vmem_frame = unsafe { Frame::from_addr(vmem) };
        map_page(vmem, vmem_frame, ptflags);
    }

    {   // Mark all text/rodata pages as read only.
        let mut remove_flags = |addr, flags| {
            if addr < 1*PDE_MAP_SIZE {
                pt0.remove_flags(addr, flags);
            } else if addr < 2*PDE_MAP_SIZE {
                pt1.remove_flags(addr, flags);
            } else if addr < 3*PDE_MAP_SIZE {
                pt2.remove_flags(addr, flags);
            } else if addr < 4*PDE_MAP_SIZE {
                pt3.remove_flags(addr, flags);
            } else {
                panic!("kernel exceeded page table allocation");
            }
        };
        let ro_start = linker_sym!(__ro_start);
        let ro_end = linker_sym!(__ro_end);
        for page in (ro_start..ro_end).step_by(PAGE_SIZE) {
            remove_flags(page, PTE_WRITABLE);
        }
    }
    
    // Map in the four page tables.
    pd.map_pagetable(0*PDE_MAP_SIZE, pt0, pdflags);
    pd.map_pagetable(1*PDE_MAP_SIZE, pt1, pdflags);
    pd.map_pagetable(2*PDE_MAP_SIZE, pt2, pdflags);
    pd.map_pagetable(3*PDE_MAP_SIZE, pt3, pdflags);

    // Construct the initial AddressSpace.
    let addrspace = AddressSpace {
        state: RWLock::new(AddressSpaceState {
            resv: FrameReserve::new()
        }),
        cr3: unsafe { pd.into_addr() }
    };

    // Enable paging.
    addrspace.activate();
    asm::enable_global_pages();
    asm::enable_paging();

    // Return the new address space!
    addrspace
}
