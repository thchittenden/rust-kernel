use core::prelude::*;
use core::{fmt, mem};
use util::asm;
logger_init!(Trace);

pub const CONFIG_ADDRESS: u16 = 0xCF8;
pub const CONFIG_DATA: u16 = 0xCFC;
pub const NOT_PRESENT: u16 = 0xFFFF;

// Common header fields.
pub const VENDOR_OFFSET: u8 = 0;
pub const DEVICE_OFFSET: u8 = 2;
pub const COMMAND_OFFSET: u8 = 4;
pub const STATUS_OFFSET: u8 = 6;
pub const REVISION_OFFSET: u8 = 8;
pub const PROG_IF_OFFSET: u8 = 9;
pub const SUBCLASS_CODE_OFFSET: u8 = 10;
pub const CLASS_CODE_OFFSET: u8 = 11;
pub const CACHE_LINE_SIZE_OFFSET: u8 = 12;
pub const LATENCY_TIEMR_OFFSET: u8 = 13;
pub const HEADER_TYPE_OFFSET: u8 = 14;
pub const BIST_OFFSET: u8 = 15;

pub const BUS_MAX: usize = 256;
pub const DEV_MAX: usize = 32;
pub const FUN_MAX: usize = 8;

#[derive(Debug)]
#[repr(C, packed)]
pub struct Header {
    pub ven_id: u16,
    pub dev_id: u16,
    pub command: u16,
    pub status: u16,
    pub rev_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8
}

impl Header {
    pub fn new (addr: PCIAddress) -> Header {
        let mut raw: [u32; 4] = [0; 4];
        for i in 0 .. 4 {
            raw[i] = read_dword(addr, 4*i as u8);
        }
        unsafe { mem::transmute(raw) }
    }
}

bitflags! {
    flags ConfigAddress: u32 {
        const ENABLE          = 0x80000000,
        const BUS_NUMBER_MASK = 0x00ff0000,
        const DEV_NUMBER_MASK = 0x0000f800,
        const FUN_NUMBER_MASK = 0x00000700,
        const REG_NUMBER_MASK = 0x000000fc,
    }
}

#[derive(Clone, Copy)]
pub struct PCIAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PCIAddress {
    pub fn new(bus: u8, device: u8, function: u8) -> PCIAddress {
        assert!(device < DEV_MAX as u8);
        assert!(function < FUN_MAX as u8);
        PCIAddress {
            bus: bus,
            device: device,
            function: function,
        }   
    }

    pub fn get_config_address(&self, offset: u8) -> ConfigAddress {
        let mut addr = ENABLE;
        addr.setmask(BUS_NUMBER_MASK, self.bus as u32);
        addr.setmask(DEV_NUMBER_MASK, self.device as u32);
        addr.setmask(FUN_NUMBER_MASK, self.function as u32);

        // The offset is already shifted by 4 so set the mask in place.
        assert!(is_aligned!(offset, 4));
        addr.setmask_inplace(REG_NUMBER_MASK, offset as u32);
        addr
    }
}

impl fmt::Debug for PCIAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}:{}:{})", self.bus, self.device, self.function)
    }
}

/// Reads a double-word from the PCI configuration space for a given device.
pub fn read_dword(addr: PCIAddress, offset: u8) -> u32 {
    let addr = addr.get_config_address(offset);
    asm::outb32(CONFIG_ADDRESS, addr.bits);
    asm::inb32(CONFIG_DATA)
}

/// Reads a word from the PCI configuration space for a given device.
pub fn read_word(addr: PCIAddress, offset: u8) -> u16 {
    assert!(is_aligned!(offset, 2));
    let dword = read_dword(addr, align!(offset, 4));
    if is_aligned!(offset, 4) {
        dword as u16
    } else {
        (dword >> 16) as u16
    }
}

/// Reads a byte from the PCI configuration space for a given device.
pub fn read_byte(addr: PCIAddress, offset: u8) -> u8 {
    let word = read_word(addr, align!(offset, 2));
    if is_aligned!(offset, 2) {
        word as u8
    } else {
        (word >> 8) as u8
    }
}

