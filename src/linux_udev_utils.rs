use super::device::*;

pub fn is_sa430(device: &udev::Device) -> bool {
    let vendor_id = get_property(device, VENDOR_ID_PROPERTY_KEY).unwrap_or_default();
    let product_id = get_property(device, PRODUCT_ID_PROPERTY_KEY).unwrap_or_default();

    vendor_id == USB_VENDOR_ID && product_id == USB_PRODUCT_ID
}

pub fn get_port(udev_device: &udev::Device) -> &str {
    udev_device
        .devnode()
        .expect("Failed to get device port")
        .to_str()
        .expect("Failed to convert device port to string")
}

pub fn get_property<'a>(udev_device: &'a udev::Device, key: &'a str) -> Option<&'a str> {
    udev_device
        .property_value(key)
        .and_then(|value| value.to_str())
}

pub fn device_from_event(udev_event: &udev::Event) -> Device {
    let udev_device = udev_event.device();
    device_from_udev(udev_device)
}

pub fn device_from_udev(udev_device: udev::Device) -> Device {
    let port = get_port(&udev_device);

    let msg = format!("Failed to get device serial number for {}", port);
    let serial_number = get_property(&udev_device, SERIAL_NUMBER_PROPERTY_KEY).expect(&msg);

    let msg = format!("Failed to get device firmware version for {}", port);
    let firmware_version = get_property(&udev_device, FIRMWARE_VERSION_PROPERTY_KEY).expect(&msg);

    Device::new(port, serial_number, firmware_version)
}
