mod bus;
mod device;
mod util;

use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::{fmt, mem};
use core::fmt::Debug;
use collections::link::{DoubleLink};
use collections::vec::Vec;
use util::{asm, KernResult};
use self::bus::PCIBus;
use self::util::{CONFIG_ADDRESS, CONFIG_DATA};
use super::{Driver, Device, DeviceClass, DeviceManager};
logger_init!(Trace);



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
    fn get_device(&self) -> KernResult<Option<Box<Device>>> { 
        let dev = try!(Box::new(PCIBus::new(CONFIG_ADDRESS, CONFIG_DATA)));
        Ok(Some(dev))
    }
    fn get_device_class(&self) -> Option<&DeviceClass> { None }
    fn create_device(&self, parent: Rc<Device>) -> KernResult<Option<Box<Device>>> { Ok(None) }
    fn _dlink(&self) -> &DoubleLink<Driver + 'static> { &self.link }
    fn _dlink_mut(&mut self) -> &mut DoubleLink<Driver + 'static> { &mut self.link }
}

/// Initializes the PCI subsystem and registers any PCI drivers with the driver registry.
pub fn init(ctx: &mut DeviceManager) {
    ctx.register_driver(Box::new(PCIDriver::new()).unwrap());
}
