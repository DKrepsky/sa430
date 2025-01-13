pub mod device;
pub mod scanner;
pub mod scanner_factory;

mod fake_scanner;
mod linux_scanner;

const USB_VENDOR_ID: &str = "2047";
const USB_PRODUCT_ID: &str = "0005";
