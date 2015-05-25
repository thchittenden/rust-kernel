#![allow(dead_code,raw_pointer_derive)]
use core::ops::Fn;

pub const MULTIBOOT_INFO_MEMORY: u32 = 0x1;
pub const MULTIBOOT_INFO_BOOTDEV: u32 = 0x2;
pub const MULTIBOOT_INFO_CMDLINE: u32 = 0x4;
pub const MULTIBOOT_INFO_MODS: u32 = 0x8;
pub const MULTIBOOT_INFO_AOUT_SYMS: u32 = 0x10;
pub const MULTIBOOT_INFO_ELF_SHDR: u32 = 0x20;
pub const MULTIBOOT_INFO_MEM_MAP: u32 = 0x40;

#[derive(Debug)]
#[repr(C, packed)]
pub struct MultibootHeader {
    flags: u32,

    // only defined if flags[0] set
    mem_lower: u32,
    mem_upper: u32,

    // only defined if flags[1] set
    boot_drive: u8,
    boot_part1: u8,
    boot_part2: u8,
    boot_part3: u8,

    // only defined if flags[2] set
    cmdline: *const u8,

    // only defined if flags[3] set
    mods_count: u32,
    mods_addr:  u32,

    // only defined if flags[5] set (ignoring flags[4], only for a.out kernels)
    syms_num:   u32,
    syms_size:  u32,
    syms_addr:  u32,
    syms_shndx: u32,

    // only defined if flags[6] set
    mmap_length: u32,
    mmap_addr:   u32,

    // only defined if flags[7] set
    drives_length: u32,
    drives_addr:   u32,

    // only defined if flags[8] set
    config_table: u32,

    // only defined if flags[9] set
    boot_loader_name: *const u8,

    // only defined if flags[10] set
    apm_table: u32,

    // only defined if flags[11] set
    vbe_control_info:  u32,
    vbe_mode_info:     u32,
    vbe_mode:          u32,
    vbe_interface_seg: u32,
    vbe_interface_off: u32,
    vbe_interface_len: u32,

}

pub const MULTIBOOT_MEMORY_AVAILABLE: u32 = 1;
pub const MULTIBOOT_MEMORY_RESERVED: u32 = 2;
pub const MULTIBOOT_MEMORY_ACPI_RECLAIMABLE: u32 = 3;
pub const MULTIBOOT_MEMORY_NVS: u32 = 4;
pub const MULTIBOOT_MEMORY_BADRAM: u32 = 5;

// The bootloader does some funky things with how the mmap array is layed out
// so there is an additional 4 bytes between each entry.
pub const MULTIBOOT_MMAP_ENTRY_OFFSET: isize = 4;

// For some reason, we can't actually print these using the derived Debug. Some
// function deep in the call chain for u64 panics about a wrong digit.
#[derive(Debug)]
#[repr(C, packed)]
pub struct MultibootMMapEntry {
    entry_size: u32,
    region_addr: u64,
    region_length: u64,
    region_type: u32,
}

impl MultibootHeader {
    
    pub fn walk_mmap<F>(&self, op: F) where F: Fn(usize, usize) {
        assert!(self.flags & MULTIBOOT_INFO_MEM_MAP != 0);

        let mmap = self.mmap_addr as *const u8;
        let mut offset: isize = 0;
        while offset < self.mmap_length as isize {
           
            // Get the current mmap entry.
            let cur_entry: &MultibootMMapEntry = unsafe {
                let addr = mmap.offset(offset) as *const MultibootMMapEntry;
                &*addr
            };

            // Make sure the memory addresses don't overflow.
            assert!(cur_entry.entry_size != 0);
            assert!(cur_entry.region_addr <= usize::max_value() as u64);
            assert!(cur_entry.region_length <= usize::max_value() as u64);
            if cur_entry.region_type == MULTIBOOT_MEMORY_AVAILABLE {
                let region_start = cur_entry.region_addr as usize;
                let region_end = region_start + cur_entry.region_length as usize;
                op(region_start, region_end);
            }

            // Go to the next entry.
            offset += cur_entry.entry_size as isize + MULTIBOOT_MMAP_ENTRY_OFFSET;
        }
    }

}
