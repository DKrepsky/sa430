/// A scanner is responsible for finding SA430 devices connected to the computer.
///
/// Use the [ScannerFactory](struct.ScannerFactory.html) to create a scanner for the current OS.
pub trait Scanner {
    /// Scans the Sa430 devices connected to the computer.
    ///
    /// # Returns
    ///
    /// A list of devices connected to the computer.
    fn scan(&self) -> Vec<Device>;
}
