struct FakeScanner {
    devices: Vec<Device>,
}

impl Scanner for FakeScanner {
    fn scan(&self) -> Vec<Device> {
        self.devices.clone()
    }
}

#[test]
fn given_a_device_is_connected_when_scan_then_print_device_information() {
    let writer = &mut Vec::new();
    let devices = vec![
        Device::new("/dev/ttyUSB1", "08FF41E50F8B3A34", "0104"),
        Device::new("/dev/ttyUSB2", "08FF41E50F8B3A35", "0104"),
        Device::new("/dev/ttyUSB3", "08FF41E50F8B3A36", "0102"),
    ];
    let scanner = FakeScanner { devices };

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
    let scanner = FakeScanner { devices };

    scan(Box::new(scanner), writer);

    let output = String::from_utf8(writer.to_vec()).unwrap();
    assert_eq!(
        output,
        "port           | serial number    | version\n\
     ---------------|------------------|--------\n\
     ---------------|------------------|--------\n"
    );
}
