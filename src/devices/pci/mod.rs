pub mod device;
mod bus;
mod util;

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::prelude::*;
use collections::link::{DoubleLink};
use util::KernResult;
use self::bus::PCIBus;
use self::util::{CONFIG_ADDRESS, CONFIG_DATA};
use super::{Driver, Device, DeviceClass, DeviceManager};

struct PCIDriver {
    link: DoubleLink<Driver + 'static>,
}

impl PCIDriver {
    fn new() -> PCIDriver {
        PCIDriver {
            link: DoubleLink::new(),
        }
    }
}

impl Driver for PCIDriver {
    fn get_name(&self) -> &str { "pcidriver" }
    fn get_device(&self) -> Option<KernResult<Box<Device>>> {
        Some(()).map(|_| {
            let dev: Box<Device> = try!(Box::new(PCIBus::new(CONFIG_ADDRESS, CONFIG_DATA)));
            Ok(dev)
        })
    }
    fn get_device_class(&self) -> Option<&DeviceClass> { None }
    fn create_device(&self, parent: Rc<Device>) -> Option<KernResult<Box<Device>>> { None }
    fn _dlink(&self) -> &DoubleLink<Driver + 'static> { &self.link }
    fn _dlink_mut(&mut self) -> &mut DoubleLink<Driver + 'static> { &mut self.link }
}

/// Initializes the PCI subsystem and registers any PCI drivers with the driver registry.
pub fn init(ctx: &mut DeviceManager) {
    ctx.register_driver(Box::new(PCIDriver::new()).unwrap());
}
