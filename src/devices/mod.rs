#![crate_name="devices"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;
extern crate collections;

mod pci;

use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::ops::Index;
use core::fmt::Debug;
use collections::hashmap::HashMap;
use collections::vec::Vec;
use util::global::Global;
logger_init!(Trace);

#[derive(Clone, Copy, Debug)]
pub enum DeviceClass {
    PCI { class: u8, subclass: u8 },
    USB { vid: u16, pid: u16 },
}

#[derive(Clone, Copy, Debug)]
pub enum DeviceID {
    TODO
}


/// A driver represents some object with information on how to initialize a device. Sub-traits of
/// Driver should have methods for creating devices from device-specific information.
pub trait Driver : Debug {
    /// Returns a device that this driver supports.
    fn get_device(&self) -> Option<Box<Device>>;

    /// Returns a collection of devices that this driver supports. This allows us to only allocate
    /// a vector if needed.
    fn get_devices(&self) -> Option<Vec<Box<Device>>>;

    /// Returns the class of the devices this driver supports.
    fn get_device_class(&self) -> Option<DeviceClass>;

    /// Tries to create a new device from the parent device that exposed a device class that this
    /// driver supports.
    fn create_device(&self, parent: Rc<Device>) -> Option<Box<Device>>;
}

/// A device is an interface for interacting with a physical device, whether it's an IDE drive or
/// an ethernet controller.
pub trait Device : HasRc + Debug {
    /// Returns an identifier for the device.
    fn get_id(&self) -> DeviceID;

    /// Returns the class of this device.
    fn get_class(&self) -> DeviceClass;

    /// Attempts to convert a device to a DeviceBus if possible. This allows us to enumerate all
    /// connected devices during driver initialization.
    fn downgrade_bus(&self) -> Option<&DeviceBus>;
}

/// A device bus is some device that allows more devices to be connected to it. If a driver
/// registers a device that is also a DeviceBus it will be enumerated and all supported devices on
/// the bus will be registered.
pub trait DeviceBus : Device {
    /// Enumerates all devices connected to the bus and inserts them into the device registry if a
    /// corresponding driver was found in the driver registry. 
    ///
    /// By passing the driver and device registries as parameters, we avoid needing to allocate
    /// extra space for returning a vector of devices.
    fn enumerate(&self, &mut DevicesCtx);
}

pub struct DevicesCtx {
    /// Map of drivers from the device classes they support.
    drivers_class_map: HashMap<DeviceClass, Box<Driver>>,

    /// Map of devices from their device classes.
    devices_class_map: HashMap<DeviceClass, Vec<Rc<Device>>>,

    /// Map of devices from their device ID's.
    devices_id_map: HashMap<DeviceID, Rc<Device>>
}

impl DevicesCtx {
    
    pub fn new() -> DevicesCtx {
        DevicesCtx {
            drivers_class_map: HashMap::new(),
            devices_class_map: HashMap::new(),
            devices_id_map: HashMap::new(),
        }
    }
    
    // Registers a driver for future use by the system.
    pub fn register_driver(&mut self, driver: Box<Driver>) {
        trace!("registering driver {:?}", &*driver);

        // Try to get any devices the driver supports.
        driver.get_device().map(|device| self.register_device(device));
        driver.get_devices().map(|devices| {
            for device in devices {
                self.register_device(device);
            }
        });

        // If the driver supports a class of device, insert it into the driver map.
        driver.get_device_class().map(|class| {

            // This sucks! Since we can't mutably borrow both self.devices_class_map and call
            // self.register_device at the same time, we have to do this horrible thing and
            // continuously lookup the value so we can release the mutable borrow on
            // self.devices_class_map. Luckily this shouldn't run often!
            let count = self.devices_class_map.lookup(class).map(Vec::len).unwrap_or(0);
            for i in 0 .. count {

                // Get a reference to the ref-counted device.
                let dev: Rc<Device> = self.devices_class_map.lookup(class).unwrap().index(i).clone();

                // Use the driver to try to create a new device and register it.
                driver.create_device(dev).map(|dev| self.register_device(dev));
            }
            
            // Insert the class into the map of classes.
            self.drivers_class_map.insert(class, driver);
        });
    }

    pub fn register_device(&mut self, device: Box<Device>) {
        trace!("registering device {:?}", &*device);

        let id = device.get_id();
        let class = device.get_class();
        
        // Insert the device into the devices_class_map and the devices_id_map.
        let rc = Rc::new(device);
        self.devices_class_map.lookup_or_insert_mut(class, || Vec::new(4).unwrap()).push(rc.clone());
        self.devices_id_map.insert(id, rc.clone());

        // If we can cast this device to a bus, enumerate it.
        if let Some(bus) = rc.downgrade_bus() {
            bus.enumerate(self);
        }
    }


}

static CTX: Global<DevicesCtx> = global_init!();

pub fn init() {
    // Initialize all drivers. These modules additionally add any modules directly
    let mut ctx = DevicesCtx::new();
    pci::init(&mut ctx);
    
    // Initialize the global context.
    CTX.init(ctx);
}
