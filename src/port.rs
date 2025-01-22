//! Struct and const definitions representing a SA430 serial port.
//!
//! This defines constants for the USB Vendor ID and Product ID of the SA430 port, as well as
//! property keys for various port attributes such as vendor ID, product ID, serial number, and firmware version.
//!
//! The `Port` struct represents an SA430 port connected to the computer, encapsulating details such as
//! the port, serial number, and firmware version. The `Port` struct provides methods for creating a new port
//! instance with specified attributes.
//!
//! # Examples
//!
//! ```rust
//! use sa430::port::Port;
//!
//! let port = Port::new("/dev/ttyUSB0", "08FF41E50F8B3A34", "0104");
//! println!("Port connected on port: {}", port.name());
//! println!("Port serial number: {}", port.serial_number());
//! println!("Port firmware version: {}", port.firmware_version());
//! ```

/// SA430 USB Vendor ID.
pub const USB_VENDOR_ID: &str = "2047";

/// SA430 USB Product ID.
pub const USB_PRODUCT_ID: &str = "0005";

/// Property name for the vendor ID of the port.
pub const VENDOR_ID_PROPERTY_KEY: &str = "ID_VENDOR_ID";

/// Property name for the product ID of the port.
pub const PRODUCT_ID_PROPERTY_KEY: &str = "ID_MODEL_ID";

/// Property name for the serial number of the port.
pub const SERIAL_NUMBER_PROPERTY_KEY: &str = "ID_SERIAL_SHORT";

/// Property name for the firmware version of the port.
pub const FIRMWARE_VERSION_PROPERTY_KEY: &str = "ID_REVISION";

/// A port represents a SA430 connected to the computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    name: String,
    serial_number: String,
    firmware_version: String,
}

impl Port {
    /// Creates a new port with the given port, serial number, and version.
    pub fn new(port: &str, serial_number: &str, firmware_version: &str) -> Self {
        Port {
            name: String::from(port),
            serial_number: String::from(serial_number),
            firmware_version: String::from(firmware_version),
        }
    }

    /// Returns the COM port name, ex "/dev/ttyUSB1".
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the serial number of the port, ex "08FF41E50F8B3A34".
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Returns the version of the port, ex "0104".
    pub fn firmware_version(&self) -> &str {
        &self.firmware_version
    }
}
