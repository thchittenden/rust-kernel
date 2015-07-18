use alloc::boxed::Box;
use alloc::rc::Rc;
use collections::link::DoubleLink;
use core::prelude::*;
use util::kernresult::*;
use ::{Driver, Device, DeviceClass, DeviceManager};
use ::pci::device::PCIDevice;

pub struct IDEDriver {
    link: DoubleLink<Driver + 'static>,
    class: DeviceClass,
}

impl IDEDriver {
    fn new() -> IDEDriver {
        IDEDriver {
            link: DoubleLink::new(),
            class: DeviceClass::PCI { class: 0x01, subclass: 0x01 },
        }
    }
}

impl Driver for IDEDriver {
    fn get_name(&self) -> &str { "idedriver" }
    fn get_device(&self) -> Option<KernResult<Box<Device>>> {
        None
    }
    fn get_device_class(&self) -> Option<&DeviceClass> {
        Some(&self.class)
    }
    fn create_device(&self, parent: Rc<Device>) -> Option<KernResult<Box<Device>>> {
        parent.as_any().downcast_ref::<PCIDevice>().map(|pciref| {
            let pci = Rc::from_ref(pciref);
            unimplemented!() 
        })
    }
    fn _dlink(&self) -> &DoubleLink<Driver + 'static> {
        &self.link
    }
    fn _dlink_mut(&mut self) -> &mut DoubleLink<Driver + 'static> {
        &mut self.link
    }
}

pub fn init(ctx: &mut DeviceManager) {
    ctx.register_driver(Box::new(IDEDriver::new()).unwrap());
}
