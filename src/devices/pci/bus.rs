use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::atomic::AtomicUsize;
use core::prelude::*;
use core::mem;
use core::any::Any;
use ::{Device, DeviceBus, DeviceClass, DeviceManager};
use super::util::*;
use super::device::PCIDevice;
use mutex::Mutex;
use util::asm;

struct BusIter<'a> {
    parent: &'a PCIBus,
    bus: usize,
    dev: usize,
    fun: usize,
}

impl<'a> BusIter<'a> {
    fn new(parent: &PCIBus) -> BusIter {
        BusIter { parent: parent, bus: 0, dev: 0, fun: 0 }
    }
}

// TODO this implementation scans every single slot. This can be improved.
impl<'a> Iterator for BusIter<'a> {
    type Item = PCIDevice; 
    fn next(&mut self) -> Option<PCIDevice> {
        let mut dev = None;
        while dev.is_none() { 
            if self.bus == BUS_MAX {
                break;
            }
           
            let addr = PCIAddress::new(self.bus as u8, self.dev as u8, self.fun as u8);
            if self.parent.is_present(addr) {
                let parent = Rc::from_ref(self.parent);
                // TODO unwrap.
                dev = Some(PCIDevice::new(addr, parent).unwrap());
            }

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

struct PCIPort {
    addr: u16,
    data: u16,
}

impl PCIPort {
    pub fn read_dword(&self, addr: PCIAddress, offset: u8) -> u32 {
        let addr = addr.get_config_address(offset);
        asm::outb32(self.addr, addr.bits());
        asm::inb32(self.data)
    }
    pub fn read_word(&self, addr: PCIAddress, offset: u8) -> u16 {
        let dword = self.read_dword(addr, align!(offset, 4));
        if is_aligned!(offset, 4) {
            dword as u16
        } else {
            (dword >> 16) as u16
        }
    }
}

pub struct PCIBus {
    rc: AtomicUsize,
    class: DeviceClass,
    port: Mutex<PCIPort>,
}

impl PCIBus {
    pub fn new(caddr: u16, cdata: u16) -> PCIBus {
        PCIBus {
            rc: AtomicUsize::new(0),
            class: DeviceClass::PCI { class: 0, subclass: 0 },
            port: Mutex::new(PCIPort {
                addr: caddr,
                data: cdata,
            }),
        }
    }

    pub fn is_present(&self, addr: PCIAddress) -> bool {
        let vid = self.port.lock().read_word(addr, VENDOR_OFFSET);
        vid != NOT_PRESENT
    }

    pub fn get_header(&self, addr: PCIAddress) -> Header {
        let port = self.port.lock();
        let mut raw: [u32; 4] = [0; 4];
        for i in 0 .. 4 {
            raw[i] = port.read_dword(addr, 4*i as u8);
        }
        unsafe { mem::transmute(raw) }
    }

    pub fn get_config(&self, addr: PCIAddress) -> DeviceConfig {
        let hdr = self.get_header(addr);
        match hdr.header_type.getmask(TYPE) {
            0x00 => {
                let port = self.port.lock();
                let mut raw: [u32; CONFIG_V0_SIZE] = [0; CONFIG_V0_SIZE];
                for i in HEADER_SIZE .. CONFIG_V0_SIZE {
                    raw[i] = port.read_dword(addr, 4*i as u8);
                }
                let mut config: ConfigV0 = unsafe { mem::transmute(raw) };
                config.hdr = hdr;
                DeviceConfig::V0(config)
            }
            0x01 => {
                let port = self.port.lock();
                let mut raw: [u32; CONFIG_V1_SIZE] = [0; CONFIG_V1_SIZE];
                for i in HEADER_SIZE .. CONFIG_V1_SIZE {
                    raw[i] = port.read_dword(addr, 4*i as u8);
                }
                let mut config: ConfigV1 = unsafe { mem::transmute(raw) };
                config.hdr = hdr;
                DeviceConfig::V1(config)
            }
            0x02 => {
                let port = self.port.lock();
                let mut raw: [u32; CONFIG_V2_SIZE] = [0; CONFIG_V2_SIZE];
                for i in HEADER_SIZE .. CONFIG_V2_SIZE {
                    raw[i] = port.read_dword(addr, 4*i as u8);
                }
                let mut config: ConfigV2 = unsafe { mem::transmute(raw) };
                config.hdr = hdr;
                DeviceConfig::V2(config)
            }
            val => panic!("unsupported pci header {}", val)
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
        "pci"
    }
    fn get_class(&self) -> &DeviceClass {
        &self.class
    }
    fn downcast_bus(&self) -> Option<&DeviceBus> {
        Some(self)
    }
    fn as_any(&self) -> &Any {
        self
    }
}

impl DeviceBus for PCIBus {
    fn enumerate(&self, ctx: &mut DeviceManager) {
        let iter = BusIter::new(self);
        for dev in iter {
            let boxed = Box::new(dev).unwrap();
            ctx.register_device(boxed).unwrap();
        }
    }
}

