//! # SA430 Crate
//!
//! This crate provides functionality to interact with the SA430 Sub-1 GHz RF Spectrum Analyzer
//! from [Texas Instruments](https://www.ti.com/). It includes modules for finding connected devices, handling
//! commands, capturing data, and more.
//!
//! ```no_run
//! use sa430::{create_scanner, Sa430};
//!
//! fn main() -> sa430::Result<()> {
//!     for port in create_scanner().scan()? {
//!         let mut device = Sa430::open(port.name())?;
//!         println!("{}: {}", port.name(), device.idn()?);
//!     }
//!     Ok(())
//! }
//! ```

pub mod channel;
pub mod device;
pub mod error;
pub mod frame;
pub mod monitor;
pub mod port;
pub mod scanner;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub(crate) mod crc;
pub(crate) mod parser;
pub(crate) mod protocol;

#[cfg(target_os = "linux")]
pub(crate) mod linux;

pub use device::Sa430;
pub use error::{Error, Result};
pub use monitor::{Event, EventHandler, Monitor};
pub use port::Port;
pub use scanner::Scanner;

/// Creates the best available scanner for the current OS.
///
/// On Linux, udev is used so the USB firmware revision is reported too. Other platforms use the cross-platform
/// [`SerialPortScanner`](scanner::SerialPortScanner).
pub fn create_scanner() -> Box<dyn Scanner> {
    #[cfg(target_os = "linux")]
    return Box::new(linux::scanner::LinuxScanner::new());

    #[cfg(not(target_os = "linux"))]
    return Box::new(scanner::SerialPortScanner::new());
}

/// Creates a connect/disconnect monitor for the current OS.
///
/// # Errors
///
/// Returns [`Error::Unsupported`] on platforms other than Linux.
pub fn create_monitor() -> Result<Box<dyn Monitor>> {
    #[cfg(target_os = "linux")]
    return Ok(Box::new(linux::monitor::LinuxMonitor::new()));

    #[cfg(not(target_os = "linux"))]
    return Err(Error::Unsupported("device monitoring"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_create_scanner_then_scan_succeeds() {
        create_scanner().scan().unwrap();
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn given_target_is_linux_when_create_monitor_then_return_a_monitor() {
        assert!(create_monitor().is_ok());
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn given_target_is_not_linux_when_create_monitor_then_return_unsupported() {
        assert!(matches!(create_monitor(), Err(Error::Unsupported(_))));
    }
}
