//! A serial port where an SA430 is connected.
//!
//! # Examples
//!
//! ```rust
//! use sa430::port::Port;
//!
//! let port = Port::new("/dev/ttyACM0", Some("08FF41E50F8B3A34"), Some("0104"));
//! println!("Port: {}", port.name());
//! println!("Serial number: {}", port.serial_number().unwrap_or("unknown"));
//! println!("Firmware version: {}", port.firmware_version().unwrap_or("unknown"));
//! ```

/// SA430 USB Vendor ID.
pub const USB_VENDOR_ID: u16 = 0x2047;

/// SA430 USB Product ID.
pub const USB_PRODUCT_ID: u16 = 0x0005;

/// A port represents an SA430 connected to the computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    name: String,
    serial_number: Option<String>,
    firmware_version: Option<String>,
}

impl Port {
    /// Creates a new port. The serial number and firmware version may not be available on every platform.
    pub fn new(name: &str, serial_number: Option<&str>, firmware_version: Option<&str>) -> Self {
        Port {
            name: name.to_owned(),
            serial_number: serial_number.map(str::to_owned),
            firmware_version: firmware_version.map(str::to_owned),
        }
    }

    /// Returns the serial port name, ex "/dev/ttyACM0" or "COM3".
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the USB serial number, ex "08FF41E50F8B3A34".
    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    /// Returns the USB firmware revision, ex "0104".
    pub fn firmware_version(&self) -> Option<&str> {
        self.firmware_version.as_deref()
    }
}
