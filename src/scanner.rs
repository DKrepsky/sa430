//! Finding SA430 devices connected to the computer.
//!
//! A [`Scanner`] searches for USB serial ports with the SA430 Vendor ID and Product ID and returns them as [`Port`]s.
//! Use [`create_scanner`](crate::create_scanner) to get the best scanner for the current OS:
//!
//! - Linux: a udev based scanner, which also reports the USB firmware revision.
//! - Other platforms: [`SerialPortScanner`], built on the cross-platform `serialport` crate.
//!
//! # Examples
//!
//! ```no_run
//! let scanner = sa430::create_scanner();
//!
//! for port in scanner.scan()? {
//!     println!("Found device at: {}", port.name());
//! }
//! # Ok::<(), sa430::Error>(())
//! ```
use serialport::{SerialPortInfo, SerialPortType};

use crate::error::Result;
use crate::port::{Port, USB_PRODUCT_ID, USB_VENDOR_ID};

/// A scanner is responsible for finding SA430 devices connected to the computer.
pub trait Scanner {
    /// Returns the ports where SA430 devices are connected.
    fn scan(&self) -> Result<Vec<Port>>;
}

/// Cross-platform scanner using the operating system's serial port enumeration.
///
/// The firmware version is not reported by this scanner.
#[derive(Debug, Default)]
pub struct SerialPortScanner;

impl SerialPortScanner {
    /// Creates a new scanner.
    pub fn new() -> Self {
        SerialPortScanner
    }
}

impl Scanner for SerialPortScanner {
    fn scan(&self) -> Result<Vec<Port>> {
        Ok(serialport::available_ports()?
            .iter()
            .filter_map(to_sa430_port)
            .collect())
    }
}

/// Returns the port if `info` describes an SA430.
fn to_sa430_port(info: &SerialPortInfo) -> Option<Port> {
    match &info.port_type {
        SerialPortType::UsbPort(usb) if usb.vid == USB_VENDOR_ID && usb.pid == USB_PRODUCT_ID => {
            Some(Port::new(&info.port_name, usb.serial_number.as_deref(), None))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serialport::UsbPortInfo;

    use super::*;

    fn a_usb_port(vid: u16, pid: u16) -> SerialPortInfo {
        SerialPortInfo {
            port_name: "COM3".into(),
            port_type: SerialPortType::UsbPort(UsbPortInfo {
                vid,
                pid,
                serial_number: Some("08FF41E50F8B3A34".into()),
                manufacturer: None,
                product: None,
            }),
        }
    }

    #[test]
    fn given_an_sa430_port_when_to_sa430_port_then_return_port() {
        let port = to_sa430_port(&a_usb_port(USB_VENDOR_ID, USB_PRODUCT_ID));
        assert_eq!(port, Some(Port::new("COM3", Some("08FF41E50F8B3A34"), None)));
    }

    #[test]
    fn given_another_usb_device_when_to_sa430_port_then_return_none() {
        assert_eq!(to_sa430_port(&a_usb_port(0x0403, 0x6001)), None);
    }

    #[test]
    fn given_a_non_usb_port_when_to_sa430_port_then_return_none() {
        let info = SerialPortInfo {
            port_name: "/dev/ttyS0".into(),
            port_type: SerialPortType::Unknown,
        };
        assert_eq!(to_sa430_port(&info), None);
    }
}
