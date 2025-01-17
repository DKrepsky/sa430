//! Struct and const definitions representing a SA430 device.
//!
//! This defines constants for the USB Vendor ID and Product ID of the SA430 device, as well as
//! property keys for various device attributes such as vendor ID, product ID, serial number, and firmware version.
//!
//! The `Device` struct represents an SA430 device connected to the computer, encapsulating details such as
//! the port, serial number, and firmware version. The `Device` struct provides methods for creating a new device
//! instance with specified attributes.
//!
//! # Examples
//!
//! ```rust
//! use sa430::device::Device;
//!
//! let device = Device::new("/dev/ttyUSB0", "08FF41E50F8B3A34", "0104");
//! println!("Device connected on port: {}", device.port());
//! println!("Device serial number: {}", device.serial_number());
//! println!("Device firmware version: {}", device.firmware_version());
//! ```

/// SA430 USB Vendor ID.
pub const USB_VENDOR_ID: &str = "2047";

/// SA430 USB Product ID.
pub const USB_PRODUCT_ID: &str = "0005";

/// Property name for the vendor ID of the device.
pub const VENDOR_ID_PROPERTY_KEY: &str = "ID_VENDOR_ID";

/// Property name for the product ID of the device.
pub const PRODUCT_ID_PROPERTY_KEY: &str = "ID_MODEL_ID";

/// Property name for the serial number of the device.
pub const SERIAL_NUMBER_PROPERTY_KEY: &str = "ID_SERIAL_SHORT";

/// Property name for the firmware version of the device.
pub const FIRMWARE_VERSION_PROPERTY_KEY: &str = "ID_REVISION";

/// A device represents a SA430 connected to the computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    port: String,
    serial_number: String,
    firmware_version: String,
}

impl Device {
    /// Creates a new device with the given port, serial number, and version.
    pub fn new(port: &str, serial_number: &str, firmware_version: &str) -> Self {
        Device {
            port: String::from(port),
            serial_number: String::from(serial_number),
            firmware_version: String::from(firmware_version),
        }
    }

    /// Returns the COM port of the device, ex "/dev/ttyUSB1".
    pub fn port(&self) -> &str {
        &self.port
    }

    /// Returns the serial number of the device, ex "08FF41E50F8B3A34".
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Returns the version of the device, ex "0104".
    pub fn firmware_version(&self) -> &str {
        &self.firmware_version
    }
}
