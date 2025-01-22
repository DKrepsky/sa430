use crate::port::*;

pub fn is_sa430(device: &udev::Device) -> bool {
    let vendor_id = get_property(device, VENDOR_ID_PROPERTY_KEY).unwrap_or_default();
    let product_id = get_property(device, PRODUCT_ID_PROPERTY_KEY).unwrap_or_default();

    vendor_id == USB_VENDOR_ID && product_id == USB_PRODUCT_ID
}

pub fn get_port(device: &udev::Device) -> &str {
    device
        .devnode()
        .expect("Failed to get device port")
        .to_str()
        .expect("Failed to convert device port to string")
}

pub fn get_property<'a>(device: &'a udev::Device, key: &'a str) -> Option<&'a str> {
    device.property_value(key).and_then(|value| value.to_str())
}

pub fn port_from_event(event: &udev::Event) -> Port {
    let device = event.device();
    port_from_device(device)
}

pub fn port_from_device(device: udev::Device) -> Port {
    let port = get_port(&device);

    let msg = format!("Failed to get device serial number for {}", port);
    let serial_number = get_property(&device, SERIAL_NUMBER_PROPERTY_KEY).expect(&msg);

    let msg = format!("Failed to get device firmware version for {}", port);
    let firmware_version = get_property(&device, FIRMWARE_VERSION_PROPERTY_KEY).expect(&msg);

    Port::new(port, serial_number, firmware_version)
}
