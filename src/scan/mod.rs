/// SA430 USB Vendor ID.
pub const USB_VENDOR_ID: &str = "2047";

/// SA430 USB Product ID.
pub const USB_PRODUCT_ID: &str = "0005";

include!("device.rs");
include!("scanner.rs");
include!("scanner_factory.rs");

#[cfg(target_os = "linux")]
include!("linux_scanner.rs");

#[cfg(test)]
mod tests;
