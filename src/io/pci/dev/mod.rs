use core::fmt;

pub mod unknown;

use alloc::boxed::Box;
use pci::{Header, DeviceAddress};

trait PCIDevice : fmt::Debug {
    
    fn new(addr: DeviceAddress, hdr: Header) -> Box<PCIDevice>;
    
}



