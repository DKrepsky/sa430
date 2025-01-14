use super::device::Device;

/// A scanner is responsible for finding SA430 devices connected to the computer.
pub trait Scanner {
    /// Scans the Sa430 devices connected to the computer.
    fn scan(&self) -> Vec<Device>;
}
