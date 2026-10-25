use crate::error::Result;
use crate::port::Port;
use crate::scanner::Scanner;

use super::udev_utils::*;

/// Scanner based on udev, which also reports the USB firmware revision.
#[derive(Debug, Default)]
pub struct LinuxScanner;

impl LinuxScanner {
    pub fn new() -> Self {
        LinuxScanner
    }
}

impl Scanner for LinuxScanner {
    fn scan(&self) -> Result<Vec<Port>> {
        let mut enumerator = udev::Enumerator::new()?;
        enumerator.match_subsystem("tty")?;

        Ok(enumerator
            .scan_devices()?
            .filter(is_sa430)
            .filter_map(|device| port_from_device(&device))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_scan_without_error() {
        let scanner = LinuxScanner::new();
        scanner.scan().unwrap();
    }
}
