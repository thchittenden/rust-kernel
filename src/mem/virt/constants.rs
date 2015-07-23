#![allow(unsigned_negation)]

/// The mask to get an entry offset (pde or pte).
pub const ENTRY_OFF_MASK: usize = 0x3ff;

/// The mask to get the offset into a frame.
pub const FRAME_OFF_MASK: usize = 0xFFF;

/// The mask to get a frame address.
pub const FRAME_NUM_MASK: usize = !FRAME_OFF_MASK;

/// How many bits we need to shift a frame number to get its offset.
pub const PT_SHIFT: usize = 12;

/// How many bits we need to shift a page table number to get its offset.
pub const PD_SHIFT: usize = 22;

/// How many bytes a page table entry maps.
pub const PTE_MAP_SIZE: usize = (1 << PT_SHIFT);

/// How many bytes a page directory entry maps.
pub const PDE_MAP_SIZE: usize = (1 << PD_SHIFT);

/// The address of the page directory in the virtual address space. This is mapped by the last page
/// table entry.
pub const PD_ADDR: usize = -PTE_MAP_SIZE;

/// The base address of all page tables in the virtual address space. This is mapped by the last
/// page directory entry.
pub const PT_BASE_ADDR: usize = -PDE_MAP_SIZE; 
