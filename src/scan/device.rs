pub const USB_VENDOR_ID: &str = "2047";
pub const USB_PRODUCT_ID: &str = "0005";

/// A device represents a SA430 connected to the computer.
#[derive(Debug, Clone)]
pub struct Device {
    port: Box<str>,
    serial: Box<str>,
    version: Box<str>,
}

impl Device {
    /// Creates a new device with the given port, serial number, and version.
    pub fn new(port: &str, serial: &str, version: &str) -> Self {
        Device {
            port: port.into(),
            serial: serial.into(),
            version: version.into(),
        }
    }

    /// Returns the port of the device, ex "/dev/ttyUSB1".
    pub fn port(&self) -> &str {
        &self.port
    }

    /// Returns the serial number of the device, ex "08FF41E50F8B3A34".
    pub fn serial(&self) -> &str {
        &self.serial
    }

    /// Returns the version of the device, ex "0104".
    pub fn version(&self) -> &str {
        &self.version
    }
}