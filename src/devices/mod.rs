//! This module defines the kernel's driver framework.
//!
//! The driver framework is built around two main notions: a Driver and a Device.
//!
//! A Driver is a software entity responsible for creating Devices. It can do so in two ways:
//!     - At Driver instantiation time, the Driver is allowed to enumerate a collection of Devices.
//!       This is useful for devices that are always connected to the computer such as a PCI bus.
//!     - When a Device is registered, if it is of a DeviceClass that the Driver supports, the
//!       Driver has a chance to create a new Device from the registered Device.
//!
//! A Device is the software interface to a physical device. Devices are allowed to register other
//! devices by supporting the DeviceBus trait. This allows devices such as PCI busses to enumerate
//! the connected devices.
//!
//! Drivers and Devices are managed by the DeviceManager. This is a global object that maintains
//! the current set of connected devices and drivers and ensures devices are properly mapped to
//! their supporting drivers.
//!

#![crate_name="devices"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude,const_fn)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;
extern crate collections;
extern crate fs;

mod pci;

use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::atomic::AtomicUsize;
use core::prelude::*;
use core::ops::Index;
use collections::link::{DoubleLink, HasDoubleLink};
use collections::linked::Linked;
use collections::hashmap::{HasKey, HashMap};
use collections::vec::Vec;
use collections::string::String;
use fs::{Path, FileCursor};
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

    /// Returns a device that this driver supports.
    fn get_device(&self) -> KernResult<Option<Box<Device>>>;

    /// Returns the class of the devices this driver supports.
    fn get_device_class(&self) -> Option<&DeviceClass>;

    /// Tries to create a new device from the parent device that exposed a device class that this
    /// driver supports.
    fn create_device(&self, parent: Rc<Device>) -> KernResult<Option<Box<Device>>>;

    /// Helper methods to allow the Driver to implement HasDoubleLink<Driver>.
    fn _dlink(&self) -> &DoubleLink<Driver + 'static>;
    fn _dlink_mut(&mut self) -> &mut DoubleLink<Driver +'static>;
}

impl HasDoubleLink<Driver> for Driver {
    fn dlink(&self) -> &DoubleLink<Driver + 'static> {
        self._dlink()
    }
    fn dlink_mut(&mut self) -> &mut DoubleLink<Driver + 'static> {
        self._dlink_mut()
    }       
}

impl HasKey<DeviceClass> for Driver {
    fn get_key(&self) -> &DeviceClass {
        self.get_device_class().unwrap()
    }
}

/// A `device` is an interface for interacting with a physical device, whether it's an IDE drive or
/// an ethernet controller.
pub trait Device : HasRc {
    /// Returns an identifier for the device.
    fn get_name(&self) -> &str;

    /// Returns the class of this device.
    fn get_class(&self) -> &DeviceClass;

    /// Attempts to convert a device to a DeviceBus if possible. This allows us to enumerate all
    /// connected devices during driver initialization. It would be great if there were a more
    /// generic way to do this but I don't there is since generic types make traits not
    /// object-safe.
    fn downcast_bus(&self) -> Option<&DeviceBus>;
}

impl HasKey<DeviceClass> for Linked<Vec<Rc<Device>>> {
    fn get_key(&self) -> &DeviceClass {
        (*self)[0].get_class()
    }
}

/// A sized wrapper for a device. This allows us to insert the device into the VFS. TODO it would
/// be great to find a way to remove this indirection.
struct DeviceWrapper {
    rc: AtomicUsize,
    device: Rc<Device>
}

impl DeviceWrapper {
    fn new(device: Rc<Device>) -> DeviceWrapper {
        DeviceWrapper {
            rc: AtomicUsize::new(0),
            device: device
        }
    }
}

impl HasRc for DeviceWrapper {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
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
    drivers_map: HashMap<DeviceClass, Driver>,

    /// Map of devices from their device classes.
    devices_map: HashMap<DeviceClass, Linked<Vec<Rc<Device>>>>,

    /// The root of the devices directory.
    root: FileCursor,
}

impl DeviceManager {
    
    pub fn new(mut root: FileCursor) -> KernResult<DeviceManager> {
        // Create directories for the devices.
        try!(root.make_node(String::from_str("devices")));
        try!(root.cd(Path::from_str("devices")));

        Ok(DeviceManager {
            drivers_map: try!(HashMap::new()),
            devices_map: try!(HashMap::new()),
            root: root,
        })
    }
    
    // Registers a driver for future use by the system.
    pub fn register_driver(&mut self, driver: Box<Driver>) {
        info!("registering driver {}", driver.get_name());


        // Try to get any devices the driver supports.
        if let Ok(Some(dev)) = driver.get_device() {
            self.register_device(dev).unwrap();
        }
        
        // If the driver supports a class of device, insert it into the driver map.
        driver.get_device_class().map(|class| *class).map(|class| {
            
            // This sucks! Since we can't mutably borrow both self.devices_map and call
            // self.register_device at the same time, we have to do this horrible thing and
            // continuously lookup the value so we can release the mutable borrow on
            // self.devices_class_map. Luckily this shouldn't run often!
            let count = self.devices_map.lookup(&class).map(|link| link.len()).unwrap_or(0);
            for i in 0 .. count {
                // Get a reference to the ref-counted device.
                let dev = self.devices_map.lookup(&class).unwrap().index(i).clone();

                // Use the driver to try to create a new device and register it.
                if let Ok(Some(dev)) = driver.create_device(dev) {
                    self.register_device(dev).unwrap();
                }
            }
            
            // Insert the class into the map of classes.
            self.drivers_map.insert(driver);
        });
    }

    pub fn register_device(&mut self, device: Box<Device>) -> KernResult<()> {
        info!("registering device {}", device.get_name());

        let class = *device.get_class();
        
        // Insert the device into the devices map.
        let rc = Rc::new(device);
        if self.devices_map.contains(&class) {
            let vec = self.devices_map.lookup_mut(&class).unwrap();
            try!(vec.push(rc.clone()))
        } else {
            let mut vec = try!(Vec::new(4).map(Linked::new).and_then(Box::new));
            try!(vec.push(rc.clone()));
            self.devices_map.insert(vec);
        }

        // Get the device name.
        let mut name = String::new();
        try!(name.append(rc.get_name()));

        // If we can cast this device to a bus, make a directory for it and enumerate into it.
        // Otherwise, register the device by name into the current root.
        if let Some(bus) = rc.downcast_bus() {
            try!(self.root.make_node(try!(name.clone())));
            try!(self.root.cd(Path::new(name)));
            bus.enumerate(self);
            try!(self.root.cd(Path::from_str("..")));
        } else {
            // Insert the device into the VFS.
            let wrapper = Rc::new(try!(Box::new(DeviceWrapper::new(rc.clone()))));
            try!(self.root.make_object(name, wrapper))
        }
        Ok(())
    }


}

static CTX: Global<DeviceManager> = Global::new();

pub fn init() {
    // Construct a directory for the device file system.
    let mut root = fs::root_cursor();
    root.make_node(String::from_str("sys"));
    root.cd(Path::new(String::from_str("sys")));
    debug!("initializing devices at {}", root.get_cd());
    
    // Create the device manager and begin enumeration.
    let mut ctx = DeviceManager::new(root).unwrap();
    pci::init(&mut ctx);
    
    // Initialize the global context.
    CTX.init(ctx);
}
