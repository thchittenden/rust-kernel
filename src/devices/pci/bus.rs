use alloc::boxed::Box;
use alloc::rc::HasRc;
use core::atomic::AtomicUsize;
use core::prelude::*;
use ::{Device, DeviceBus, DeviceClass, DeviceManager};
use super::util::*;
use super::device::PCIDevice;
logger_init!(Trace);

struct BusIter {
    bus: usize,
    dev: usize,
    fun: usize,
}

impl BusIter {
    fn new() -> BusIter {
        BusIter { bus: 0, dev: 0, fun: 0 }
    }
}

// TODO this implementation scans every single slot. This can be improved.
impl Iterator for BusIter {
    type Item = PCIDevice; 
    fn next(&mut self) -> Option<PCIDevice> {
        let mut dev = None;
        while dev.is_none() { 
            if self.bus == BUS_MAX {
                break;
            }
           
            let addr = PCIAddress::new(self.bus as u8, self.dev as u8, self.fun as u8);
            let hdr = scan_device(addr);
            dev = hdr.map(|hdr| PCIDevice::new(addr, hdr).unwrap());

            // Increment our index.
            self.fun += 1;
            if self.fun == FUN_MAX {
                self.fun = 0;
                self.dev += 1;
                if self.dev == DEV_MAX {
                    self.dev = 0;
                    self.bus += 1;
                }
            }
        }
        dev
    }
}

/// Checks if there is a device at the given PCI address.
fn scan_device(addr: PCIAddress) -> Option<Header> {
    let vendor_id = read_word(addr, VENDOR_OFFSET);
    if vendor_id != NOT_PRESENT {
        let hdr = Header::new(addr);
        info!("detected PCI device at ({:?}): {:?}", addr, hdr);
        Some(hdr)
    } else {
        None
    }
}

pub struct PCIBus {
    rc: AtomicUsize,
    class: DeviceClass,
}

impl PCIBus {
    pub fn new() -> PCIBus {
        PCIBus {
            rc: AtomicUsize::new(0),
            class: DeviceClass::PCI { class: 0, subclass: 0 },
        }
    }
}

impl HasRc for PCIBus {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}  

impl Device for PCIBus {
    fn get_name(&self) -> &str {
        "pcibus"
    }
    fn get_class(&self) -> &DeviceClass {
        &self.class
    }
    fn downcast_bus(&self) -> Option<&DeviceBus> {
        Some(self)
    }
}

impl DeviceBus for PCIBus {
    fn enumerate(&self, ctx: &mut DeviceManager) {
        let iter = BusIter::new();
        for dev in iter {
            let boxed = Box::new(dev).unwrap();
            ctx.register_device(boxed);
        }
    }
}

