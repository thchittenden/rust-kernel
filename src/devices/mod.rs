#![crate_name="devices"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude,const_fn)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;
extern crate collections;

//mod pci;

use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::ops::Index;
use collections::linked::Linked;
use collections::hashmap::{HasKey, HashMap};
use collections::vec::Vec;
use util::global::Global;
use util::KernResult;
logger_init!(Trace);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeviceClass {
    PCI { class: u8, subclass: u8 },
    USB { vid: u16, pid: u16 },
}

/// A driver represents some object with information on how to initialize a device. Sub-traits of
/// Driver should have methods for creating devices from device-specific information.
pub trait Driver {
    /// Returns a name for this driver.
    fn get_name(&self) -> &str;

    /// Returns a device that this driver supports. We need to return a linked device because
    /// there's no way to make HasSingleLink a trait bound on Driver :(
    fn get_device(&self) -> Option<Box<Device>>;

    /// Returns a collection of devices that this driver supports. This allows us to only allocate
    /// a vector if needed.
    fn get_devices(&self) -> Option<Vec<Box<Device>>>;

    /// Returns the class of the devices this driver supports.
    fn get_device_class(&self) -> Option<DeviceClass>;
    fn borrow_device_class(&self) -> Option<&DeviceClass>;

    /// Tries to create a new device from the parent device that exposed a device class that this
    /// driver supports.
    fn create_device(&self, parent: Rc<Device>) -> Option<Box<Device>>;
}

impl HasKey<DeviceClass> for Linked<Driver> {
    fn get_key(&self) -> &DeviceClass {
        self.borrow_device_class().unwrap()
    }
}

/// A device is an interface for interacting with a physical device, whether it's an IDE drive or
/// an ethernet controller.
pub trait Device : HasRc {
    /// Returns an identifier for the device.
    fn get_name(&self) -> &str;

    /// Returns the class of this device.
    fn get_class(&self) -> DeviceClass;
    fn borrow_class(&self) -> &DeviceClass;

    /// Attempts to convert a device to a DeviceBus if possible. This allows us to enumerate all
    /// connected devices during driver initialization. It would be great if there were a more
    /// generic way to do this but I don't there is since generic types make traits not
    /// object-safe.
    fn downgrade_bus(&self) -> Option<&DeviceBus>;
}

impl HasKey<DeviceClass> for Linked<Vec<Rc<Device>>> {
    fn get_key(&self) -> &DeviceClass {
        (*self)[0].borrow_class()
    }
}

/// A device bus is some device that allows more devices to be connected to it. If a driver
/// registers a device that is also a DeviceBus it will be enumerated and all supported devices on
/// the bus will be registered.
pub trait DeviceBus : Device {
    /// Enumerates all devices on the bus and registers them with the current context.
    fn enumerate(&self, &mut DeviceManager);
}

pub struct DeviceManager {
    /// Map of drivers from the device classes they support.
    drivers_map: HashMap<DeviceClass, Linked<Driver>>,

    /// Map of devices from their device classes.
    devices_map: HashMap<DeviceClass, Linked<Vec<Rc<Device>>>>,
}

impl DeviceManager {
    
    pub fn new() -> KernResult<DeviceManager> {
        Ok(DeviceManager {
            drivers_map: try!(HashMap::new()),
            devices_map: try!(HashMap::new()),
        })
    }
    
    // Registers a driver for future use by the system.
    pub fn register_driver(&mut self, driver: Box<Linked<Driver>>) {
        info!("registering driver {}", driver.get_name());

        // Try to get any devices the driver supports.
        driver.get_device().map(|device| self.register_device(device));
        driver.get_devices().map(|devices| {
            for device in devices {
                self.register_device(device);
            }
        });

        // If the driver supports a class of device, insert it into the driver map.
        driver.get_device_class().map(|class| {

            // This sucks! Since we can't mutably borrow both self.devices_map and call
            // self.register_device at the same time, we have to do this horrible thing and
            // continuously lookup the value so we can release the mutable borrow on
            // self.devices_class_map. Luckily this shouldn't run often!
            let count = self.devices_map.lookup(&class).map(|link| (*link).len()).unwrap_or(0);
            for i in 0 .. count {
                // Get a reference to the ref-counted device.
                let dev: Rc<Device> = self.devices_map.lookup(&class).unwrap().index(i).clone();

                // Use the driver to try to create a new device and register it.
                driver.create_device(dev).map(|dev| self.register_device(dev));
            }
            
            // Insert the class into the map of classes.
            self.drivers_map.insert(driver);
        });
    }

    pub fn register_device(&mut self, device: Box<Device>) {
        info!("registering device {}", device.get_name());

        let class = device.get_class();
        
        // Insert the device into the devices_class_map and the devices_id_map.
        let rc = Rc::new(device);
        let res = self.devices_map.lookup_or_insert_mut(&class, || {
            let vec = Vec::new(4).unwrap();
            Box::new(Linked::new(vec)).unwrap()
        }).push(rc.clone());
        assert!(res.is_ok());

        // If we can cast this device to a bus, enumerate it.
        if let Some(bus) = rc.downgrade_bus() {
            bus.enumerate(self);
        }
    }


}

static CTX: Global<DeviceManager> = Global::new();

pub fn init() {
    debug!("initializing devices");
    // Initialize all drivers. These modules additionally add any modules directly
    let mut ctx = DeviceManager::new().unwrap();
    //pci::init(&mut ctx);
    
    // Initialize the global context.
    CTX.init(ctx);
}
