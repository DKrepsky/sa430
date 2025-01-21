//! # SA430 Crate
//!
//! This crate provides functionality to interact with SA430 SA430 Sub-1 GHz RF Spectrum Analyzer
//! from [Texas Instruments](https://www.ti.com/). It includes modules for scanning devices, handling
//! commands, capturing data, and more.

pub mod device;
pub mod monitor;
pub mod scanner;

#[cfg(target_os = "linux")]
pub(crate) mod linux;

/// Creates a scanner for the current OS.
pub fn create_scanner() -> Box<dyn scanner::Scanner> {
    #[cfg(target_os = "linux")]
    return Box::new(linux::scanner::LinuxScanner::new());

    #[cfg(not(target_os = "linux"))]
    panic!("No scanner for current OS");
}

/// Creates a monitor for the current OS.
pub fn create_monitor() -> Box<dyn monitor::Monitor> {
    #[cfg(target_os = "linux")]
    return Box::new(linux::monitor::LinuxMonitor::new());

    #[cfg(not(target_os = "linux"))]
    panic!("No monitor for current OS");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn given_target_is_linux_when_create_scanner_then_create_a_linux_scanner() {
        create_scanner();
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    #[should_panic]
    fn given_target_is_unknown_when_create_scanner_then_panic() {
        create_monitor();
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn given_target_is_linux_when_create_monitor_then_create_a_linux_monitor() {
        create_monitor();
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    #[should_panic]
    fn given_target_is_unknown_when_create_monitor_then_panic() {
        create_monitor();
    }
}
