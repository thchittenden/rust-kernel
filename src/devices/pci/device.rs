use alloc::rc::HasRc;
use core::atomic::AtomicUsize;
use core::prelude::*;
use core::fmt::Write;
use collections::string::String;
use super::util::{Header, PCIAddress};
use util::KernResult;
use ::{Device, DeviceBus, DeviceClass};

pub struct PCIDevice {
    rc: AtomicUsize,
    name: String,
    address: PCIAddress,
    class: DeviceClass
}

impl PCIDevice {
    pub fn new(addr: PCIAddress, header: Header) -> KernResult<PCIDevice> {
        let mut name = String::new();
        try!(write!(name, "pci{:02x}:{:02x}:{:02x}", addr.bus, addr.device, addr.function));
        Ok(PCIDevice {
            rc: AtomicUsize::new(0),
            name: name,
            address: addr,
            class: DeviceClass::PCI { class: header.class, subclass: header.subclass },
        })
    }
}

impl HasRc for PCIDevice {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}

impl Device for PCIDevice {
    fn get_name(&self) -> &str {
        self.name.as_str() 
    }
    fn get_class(&self) -> &DeviceClass {
        &self.class
    }
    fn downcast_bus(&self) -> Option<&DeviceBus> {
        None
    }
}
