//! Contains the core components for scanning connected Sa430 devices.
//!
//! A [Scanner](trait.Scanner.html) will search for USB devices connected to the computer with
//! a specific USB Vendor ID and Product ID. It will then return a list of [Port](struct.Port.html)
//! that represents the port where the SA430 device is connected to.
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
//! for port in scanner.scan() {
//!     println!("Found device at: {:?}", port);
//! }
//! ```
use super::port::Port;

/// A scanner is responsible for finding SA430 devices connected to the computer.
///
/// The scanner will search for devices with a specific USB Vendor ID and Product ID and return the ports where the
/// devices are connected.
pub trait Scanner {
    /// Scans the Sa430 devices connected to the computer.
    ///
    /// # Returns
    ///
    /// A list of ports were SA430 devices are connected to.
    fn scan(&self) -> Vec<Port>;
}
