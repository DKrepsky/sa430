/// This module contains the core components for the scanning functionality.
///
/// A [Scanner](trait.Scanner.html) will search for USB devices connected to the computer with
/// a specific USB Vendor ID and Product ID. It will then return a list of [Device](struct.Device.html)
/// that represents the SA430 devices connected to the computer.
///
/// Use the [ScannerFactory](struct.ScannerFactory.html) to create a scanner for the
/// current OS and scan for devices.
///
/// # Examples
///
/// ```rust
/// use sa430::scan::ScannerFactory;
///
/// let result = ScannerFactory::create();
///
/// if let Some(scanner) = result.ok() {
///     for device in scanner.scan() {
///         println!("Found device: {:?}", device);
///     }
/// } else {
///    println!("No scanner for current OS");
/// }
/// ```
pub mod scan;
