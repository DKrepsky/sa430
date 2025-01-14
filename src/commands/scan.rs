#![allow(unused_imports)]
use crate::fixtures::FakeScanner;
use sa430::device::Device;
use sa430::scanner::Scanner;

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

#[test]
fn given_a_device_is_connected_when_scan_then_print_device_information() {
    let writer = &mut Vec::new();
    let devices = vec![
        Device::new("/dev/ttyUSB1", "08FF41E50F8B3A34", "0104"),
        Device::new("/dev/ttyUSB2", "08FF41E50F8B3A35", "0104"),
        Device::new("/dev/ttyUSB3", "08FF41E50F8B3A36", "0102"),
    ];
    let scanner = FakeScanner::new(devices);

    scan(Box::new(scanner), writer);

    let output = String::from_utf8(writer.to_vec()).unwrap();
    assert_eq!(
        output,
        "port           | serial number    | version\n\
         ---------------|------------------|--------\n\
         /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n\
         /dev/ttyUSB2   | 08FF41E50F8B3A35 | 0104\n\
         /dev/ttyUSB3   | 08FF41E50F8B3A36 | 0102\n\
         ---------------|------------------|--------\n"
    );
}

#[test]
fn given_no_device_is_connected_when_scan_then_print_no_device_information() {
    let writer = &mut Vec::new();
    let devices = vec![];
    let scanner = FakeScanner::new(devices);

    scan(Box::new(scanner), writer);

    let output = String::from_utf8(writer.to_vec()).unwrap();
    assert_eq!(
        output,
        "port           | serial number    | version\n\
         ---------------|------------------|--------\n\
         ---------------|------------------|--------\n"
    );
}
