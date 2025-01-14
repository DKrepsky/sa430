use sa430::scan::*;

/// Handles the scan command logic.
///
/// Will scan for devices using the provided scanner and print a list of devices found using the provided writer.
///
/// # Arguments
/// * `scanner` - The scanner to use to find the devices.
/// * `writer` - The writer to output the devices found.
///
pub fn scan(scanner: Box<dyn Scanner>, writer: &mut dyn std::io::Write) {
    let devices = scanner.scan();
    writeln!(writer, "port           | serial number    | version").unwrap();
    writeln!(writer, "---------------|------------------|--------").unwrap();
    for device in devices {
        print_device_information(&device, writer);
    }
    writeln!(writer, "---------------|------------------|--------").unwrap();
}

fn print_device_information(device: &Device, writer: &mut dyn std::io::Write) {
    writeln!(
        writer,
        "{:14} | {:16} | {:4}",
        device.port(),
        device.serial(),
        device.version()
    )
    .unwrap();
}
