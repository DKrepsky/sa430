use udev::Enumerator;

use crate::device::Device;
use crate::scanner::Scanner;

use super::udev_utils::*;

pub struct LinuxScanner;

impl LinuxScanner {
    pub fn new() -> Self {
        LinuxScanner
    }

    fn enumerator(&self) -> Enumerator {
        let mut enumerator = Enumerator::new().expect("Failed to create udev enumerator");

        enumerator
            .match_subsystem("tty")
            .expect("Failed to match tty subsystem");

        enumerator
    }
}

impl Scanner for LinuxScanner {
    fn scan(&self) -> Vec<Device> {
        return self
            .enumerator()
            .scan_devices()
            .expect("Failed to scan devices")
            .filter(is_sa430)
            .map(device_from_udev)
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_scan_without_panicking() {
        let scanner = LinuxScanner::new();
        scanner.scan();
    }
}
