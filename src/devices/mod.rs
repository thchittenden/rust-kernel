mod pci;

enum DeviceClass {
    PCI { class: u8, subclass: u8 },
    USB { vid: u16, pid: u16 },
}

enum DeviceID {
    
}


/// A driver represents some object with information on how to initialize a device. Sub-traits of
/// Driver should have methods for creating devices from device-specific information.
trait Driver {
    /// Returns the class of the devices this driver supports.
    fn get_device_class(&self) -> DeviceClass;

    /// Attempts to convert a reference to a Driver to a reference to a more specialized type. This
    /// is analogous to Any's downgrade method. If a device supports a particular class (PCI, USB),
    /// it should be possible to downgrade the device to the more specialized class Driver class
    /// that will allow instantiating devices of that type.
    fn downgrade<T: Driver>(&self) -> Option<&T>;
}

/// A device is an interface for interacting with a physical device, whether it's an IDE drive or
/// an ethernet controller.
trait Device {

    /// Returns an identifier for the device.
    fn get_device_id(&self) -> DeviceID;

}

/// A device bus is some device that allows more devices to be connected to it. During start-up
/// DeviceBus's are queried for all connected devices.
trait DeviceBus : Device {
    /// Enumerates all devices connected to the bus and inserts them into the device registry if a
    /// corresponding driver was found in the driver registry. 
    ///
    /// By passing the driver and device registries as parameters, we avoid needing to allocate
    /// extra space for returning a vector of devices.
    fn enumerate(&self, &mut DriverRegistry, &mut DeviceRegistry);
}

struct DriverRegistry {
    drivers: HashMap<DeviceClass, Box<Driver>>
}

struct DeviceRegistry {
    devices: HashMap<DeviceID, Box<Device>>
}

static DRIVERS: Global<DriverRegistry> = global_init!();
static DEVICES: Global<DeviceRegistry> = global_init!();

pub fn init() {
    let drivers = DriverRegistry::new();
    pci::init(&mut drivers);
    
    let devices = DeviceRegistry::new();
    
    DRIVERS.init(drivers);
    DEVICES.init(devices);
}
