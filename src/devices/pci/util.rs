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

// Until mem::size_of is const, this is magic. These are not byte counts but number of u32's.
pub const HEADER_SIZE: usize = 4;
pub const CONFIG_V0_SIZE: usize = 16;
pub const CONFIG_V1_SIZE: usize = 16;
pub const CONFIG_V2_SIZE: usize = 18;

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
    pub header_type: HeaderType,
    pub bist: u8
}

pub enum DeviceConfig {
    V0(ConfigV0),
    V1(ConfigV1),
    V2(ConfigV2),
}

impl DeviceConfig {
    pub fn get_header(&self) -> &Header {
        match self {
            &DeviceConfig::V0(ref cfg) => &cfg.hdr,
            &DeviceConfig::V1(ref cfg) => &cfg.hdr,
            &DeviceConfig::V2(ref cfg) => &cfg.hdr,
        }
    }
}

/// The configuration area when the header type is 0x00. This corresponds to a PCI device.
#[derive(Debug)]
#[repr(C, packed)]
pub struct ConfigV0 {
    pub hdr: Header,
    pub bar0: u32,
    pub bar1: u32,
    pub bar2: u32,
    pub bar3: u32,
    pub bar4: u32,
    pub bar5: u32,
    pub cardbus_cis_ptr: u32,
    pub subsys_vid: u16,
    pub subsys_pid: u16,
    pub exrom_bar: u32,
    pub capabilities: u8,
    reserved0: u8,
    reserved1: u16,
    reserved2: u32,
    pub int_line: u8, // Which PIC IRQ the device uses.
    pub int_pin: u8,  // Which APIC line the devices uses.
    pub min_grant: u8, 
    pub max_lat: u8,
}

#[repr(C, packed)]
pub struct ConfigV1 {
    pub hdr: Header,
    pub bar0: u32,
    pub bar1: u32,
    pub primary_bus_number: u8,
    pub secondary_bus_number: u8,
    pub subordinate_bus_number: u8,
    pub secondary_lat_timer: u8,
    pub io_base: u8,
    pub io_limit: u8,
    pub secondary_status: u16,
    pub mem_base: u16,
    pub mem_limit: u16,
    pub prefetch_base: u16,
    pub prefetch_limit: u16,
    pub prefetch_base_upper: u32,
    pub prefetch_limit_upper: u32,
    pub io_base_upper: u16,
    pub io_limit_upper: u16,
    pub capabilities: u8,
    reserved0: u8,
    reserved1: u16,
    pub exrom_bar: u32,
    pub int_line: u8,
    pub int_pin: u8,
    pub bridge_ctl: u16,
}

#[repr(C, packed)]
pub struct ConfigV2 {
    pub hdr: Header,
    pub cardbus_bar: u32,
    pub capabilities: u8,
    reserved0: u8,
    pub secondary_status: u16,
    pub pci_bus_num: u8,
    pub cardbus_bus_num: u8,
    pub subordinate_bus_num: u8,
    pub cardbus_lat_timer: u8,
    pub mem_base0: u32,
    pub mem_limit0: u32,
    pub mem_base1: u32,
    pub mem_limit1: u32,
    pub io_base0: u32,
    pub io_limit0: u32,
    pub io_base1: u32,
    pub io_limit1: u32,
    pub int_line: u8,
    pub int_pin: u8,
    pub bridge_ctl: u16,
    pub subsys_dev_id: u16,
    pub subsis_ven_id: u16,
    pub legacy_base: u32
}

bitflags! {
    #[derive(Debug)]
    flags HeaderType: u8 {
        const MULTIFUNCTION = 0x80,
        const TYPE          = 0x03,
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

