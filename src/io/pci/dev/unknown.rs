use core::fmt;
use alloc::boxed::Box;
use pci::{Header, DeviceAddress};
use pci::dev::PCIDevice;


struct UnknownDevice {
    address: DeviceAddress,
    header: Header
}

impl PCIDevice for UnknownDevice {

    fn new(addr: DeviceAddress, hdr: Header) -> Box<PCIDevice> {
        Box::new(UnknownDevice {
            address: addr,
            header: hdr,
        }).unwrap() as Box<PCIDevice>
    }

}

impl fmt::Debug for UnknownDevice {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnknownDevice@({:?}) {{ class: {}, subclass: {} }}", 
            self.address, self.header.class, self.header.subclass);
    }

}

