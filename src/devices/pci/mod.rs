mod dev;

use core::prelude::*;
use core::mem;
use core::fmt;
use util::asm;
logger_init!(Trace);

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;
const NOT_PRESENT: u16 = 0xFFFF;

// Common header fields.
const VENDOR_OFFSET: u8 = 0;
const DEVICE_OFFSET: u8 = 2;
const COMMAND_OFFSET: u8 = 4;
const STATUS_OFFSET: u8 = 6;
const REVISION_OFFSET: u8 = 8;
const PROG_IF_OFFSET: u8 = 9;
const SUBCLASS_CODE_OFFSET: u8 = 10;
const CLASS_CODE_OFFSET: u8 = 11;
const CACHE_LINE_SIZE_OFFSET: u8 = 12;
const LATENCY_TIEMR_OFFSET: u8 = 13;
const HEADER_TYPE_OFFSET: u8 = 14;
const BIST_OFFSET: u8 = 15;

#[derive(Debug)]
#[repr(C, packed)]
struct Header {
    ven_id: u16,
    dev_id: u16,
    command: u16,
    status: u16,
    rev_id: u8,
    prog_if: u8,
    subclass: u8,
    class: u8,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8
}

impl Header {
    fn new (addr: DeviceAddress) -> Header {
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
struct DeviceAddress {
    bus: u8,
    device: u8,
    function: u8,
}

impl DeviceAddress {
    pub fn new(bus: u8, device: u8, function: u8) -> DeviceAddress {
        DeviceAddress {
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

impl fmt::Debug for DeviceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}:{}:{})", self.bus, self.device, self.function)
    }
}

/// Reads a double-word from the PCI configuration space for a given device.
fn read_dword(addr: DeviceAddress, offset: u8) -> u32 {
    let addr = addr.get_config_address(offset);
    asm::outb32(CONFIG_ADDRESS, addr.bits);
    asm::inb32(CONFIG_DATA)
}

/// Reads a word from the PCI configuration space for a given device.
fn read_word(addr: DeviceAddress, offset: u8) -> u16 {
    assert!(is_aligned!(offset, 2));
    let dword = read_dword(addr, align!(offset, 4));
    if is_aligned!(offset, 4) {
        dword as u16
    } else {
        (dword >> 16) as u16
    }
}

/// Reads a byte from the PCI configuration space for a given device.
fn read_byte(addr: DeviceAddress, offset: u8) -> u8 {
    let word = read_word(addr, align!(offset, 2));
    if is_aligned!(offset, 2) {
        word as u8
    } else {
        (word >> 8) as u8
    }
}

/*
fn find_driver(hdr: Header) -> Box<PCIDevice> {
    match (hdr.class, hdr.subclass) {
        (0x01, 0x01) => Box::new(IDEDevice::new(addr, hdr)),
        _ => Box::new(UnknownDevice::new(addr, hdr))
    }
}
*/

/// Checks if there is a device at the given PCI address.
fn scan_device(addr: DeviceAddress)  {
    let vendor_id = read_word(addr, VENDOR_OFFSET);
    if vendor_id != NOT_PRESENT {
        let hdr = Header::new(addr);
        info!("detected PCI device at ({:?}): {:?}", addr, hdr);
    }
}

/// Scans all PCI slots for active devices. This should definitely be improved!
pub fn scan_bus() {
    for bus in 0 .. 256 {
        for dev in 0 .. 32 {
            for fun in 0 .. 8 {
                scan_device(DeviceAddress::new(bus as u8, dev as u8, fun as u8));
            }
        }
    }
}


/// Initializes the PCI subsystem.
pub fn init() {
    
      

}
