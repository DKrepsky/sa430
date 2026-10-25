use crate::port::*;

/// udev property holding the USB vendor ID.
const VENDOR_ID_PROPERTY_KEY: &str = "ID_VENDOR_ID";

/// udev property holding the USB product ID.
const PRODUCT_ID_PROPERTY_KEY: &str = "ID_MODEL_ID";

/// udev property holding the USB serial number.
const SERIAL_NUMBER_PROPERTY_KEY: &str = "ID_SERIAL_SHORT";

/// udev property holding the USB device revision (firmware version).
const FIRMWARE_VERSION_PROPERTY_KEY: &str = "ID_REVISION";

pub fn is_sa430(device: &udev::Device) -> bool {
    let id = |key| get_property(device, key).and_then(|value| u16::from_str_radix(value, 16).ok());

    id(VENDOR_ID_PROPERTY_KEY) == Some(USB_VENDOR_ID) && id(PRODUCT_ID_PROPERTY_KEY) == Some(USB_PRODUCT_ID)
}

fn get_property<'a>(device: &'a udev::Device, key: &str) -> Option<&'a str> {
    device.property_value(key).and_then(|value| value.to_str())
}

/// Builds a port from a udev device. Returns `None` if the device has no device node.
pub fn port_from_device(device: &udev::Device) -> Option<Port> {
    let name = device.devnode()?.to_str()?;

    Some(Port::new(
        name,
        get_property(device, SERIAL_NUMBER_PROPERTY_KEY),
        get_property(device, FIRMWARE_VERSION_PROPERTY_KEY),
    ))
}
