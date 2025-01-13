use sa430::scanner::device::Device;
use sa430::scanner::scanner::Scanner;
#[allow(unused_imports)]
use sa430::scanner::scanner_factory;

pub fn scan(scanner: Box<dyn Scanner>, writer: &mut dyn std::io::Write) {
    let devices = scanner.scan();
    writeln!(writer, "port           | serial number    | version").unwrap();
    writeln!(writer, "---------------|------------------|--------").unwrap();
    for device in devices {
        print_device_information(&device, writer);
    }
    println!("---------------|------------------|--------");
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
    let scanner = scanner_factory::create("ci");
    scan(scanner, writer);

    let output = String::from_utf8(writer.to_vec()).unwrap();
    assert_eq!(
        output,
        "port           | serial number    | version\n\
         ---------------|------------------|--------\n\
         /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n\
         /dev/ttyUSB2   | 08FF41E50F8B3A35 | 0104\n\
         /dev/ttyUSB3   | 08FF41E50F8B3A36 | 0102\n"
    );
}
