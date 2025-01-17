//! Contains the core components for scanning connected Sa430 devices.
//!
//! A [Scanner](trait.Scanner.html) will search for USB devices connected to the computer with
//! a specific USB Vendor ID and Product ID. It will then return a list of [Device](struct.Device.html)
//! that represents the SA430 devices connected to the computer.
//!
//! Use the [ScannerFactory](struct.ScannerFactory.html) to create a scanner for the
//! current OS and scan for devices.
//!
//! # Examples
//!
//! ```ignore
//! use sa430::create_scanner;
//!
//! let scanner = create_scanner();
//!
//! for device in scanner.scan() {
//!     println!("Found device: {:?}", device);
//! }
//! ```
use super::device::Device;

/// A scanner is responsible for finding SA430 devices connected to the computer.
pub trait Scanner {
    /// Scans the Sa430 devices connected to the computer.
    ///
    /// # Returns
    ///
    /// A list of devices connected to the computer.
    fn scan(&self) -> Vec<Device>;
}
