use alloc::rc::{Rc, HasRc};
use core::atomic::AtomicUsize;
use core::prelude::*;
use core::fmt::Write;
use core::any::Any;
use collections::string::String;
use super::util::{DeviceConfig, PCIAddress};
use super::bus::PCIBus;
use util::KernResult;
use ::{Device, DeviceBus, DeviceClass};

pub struct PCIDevice {
    rc: AtomicUsize,
    name: String,
    bus: Rc<PCIBus>,
    address: PCIAddress,
    class: DeviceClass,
    config: DeviceConfig,
}

impl PCIDevice {
    pub fn new(addr: PCIAddress, bus: Rc<PCIBus>) -> KernResult<PCIDevice> {

        // Construct the name of the device.
        let mut name = String::new();
        try!(write!(name, "pci{:02x}:{:02x}:{:02x}", addr.bus, addr.device, addr.function));

        // Extract the device configuration.
        let config = bus.get_config(addr);
        let class = DeviceClass::PCI { 
            class: config.get_header().class,
            subclass: config.get_header().subclass,
        };

        Ok(PCIDevice {
            rc: AtomicUsize::new(0),
            name: name,
            bus: bus,
            address: addr,
            class: class,
            config: config,
        })
    }

    pub fn get_config(&self) -> &DeviceConfig {
        &self.config
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
    fn as_any(&self) -> &Any {
        self
    }
}
