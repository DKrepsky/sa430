use super::scanner::*;

use udev::Enumerator;

#[derive(Default)]
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
            .map(from_udev_to_device)
            .collect();
    }
}

fn is_sa430(device: &udev::Device) -> bool {
    if let Some(vendor_id) = device.property_value("ID_VENDOR_ID") {
        if let Some(product_id) = device.property_value("ID_MODEL_ID") {
            return vendor_id == USB_VENDOR_ID && product_id == USB_PRODUCT_ID;
        }
    }
    false
}

fn from_udev_to_device(device: udev::Device) -> Device {
    Device::new(
        port_of(&device),
        serial_number_of(&device),
        version_of(&device),
    )
}

fn port_of(device: &udev::Device) -> &str {
    device
        .devnode()
        .expect("Device node not found")
        .to_str()
        .expect("Device node does not contain valid UTF-8 port")
}

fn serial_number_of(device: &udev::Device) -> &str {
    device
        .property_value("ID_SERIAL_SHORT")
        .expect("Serial number not found")
        .to_str()
        .expect("Serial number does not contain valid UTF-8 string")
}

fn version_of(device: &udev::Device) -> &str {
    device
        .property_value("ID_REVISION")
        .expect("Version not found")
        .to_str()
        .expect("Version does not contain valid UTF-8 string")
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
