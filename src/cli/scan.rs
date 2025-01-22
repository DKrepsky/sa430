use sa430::port::Port;
use sa430::scanner::Scanner;

/// Handles the scan command logic.
///
/// Will scan for devices using the provided scanner and print a list of ports where the devices were found using the
/// provided writer.
///
/// # Arguments
/// * `scanner` - The scanner to use to find the devices.
/// * `writer` - The writer to output the devices found.
///
pub fn scan(scanner: Box<dyn Scanner>, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let ports = scanner.scan();
    writeln!(writer, "port           | serial number    | version")?;
    writeln!(writer, "---------------|------------------|--------")?;
    for port in ports {
        print_port_information(&port, writer)?;
    }
    writeln!(writer, "---------------|------------------|--------")
}

fn print_port_information(port: &Port, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writeln!(
        writer,
        "{:14} | {:16} | {:4}",
        port.name(),
        port.serial_number(),
        port.firmware_version()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeScanner {
        ports: Vec<Port>,
    }

    impl Scanner for FakeScanner {
        fn scan(&self) -> Vec<Port> {
            self.ports.clone()
        }
    }

    #[test]
    fn given_a_device_is_connected_when_scan_then_print_port_information() {
        let writer = &mut Vec::new();
        let ports = vec![
            Port::new("/dev/ttyUSB1", "08FF41E50F8B3A34", "0104"),
            Port::new("/dev/ttyUSB2", "08FF41E50F8B3A35", "0104"),
            Port::new("/dev/ttyUSB3", "08FF41E50F8B3A36", "0102"),
        ];
        let scanner = FakeScanner { ports };

        scan(Box::new(scanner), writer).unwrap();

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
    fn given_no_device_is_connected_when_scan_then_print_no_port_information() {
        let writer = &mut Vec::new();
        let ports = vec![];
        let scanner = FakeScanner { ports };

        scan(Box::new(scanner), writer).unwrap();

        let output = String::from_utf8(writer.to_vec()).unwrap();
        assert_eq!(
            output,
            "port           | serial number    | version\n\
         ---------------|------------------|--------\n\
         ---------------|------------------|--------\n"
        );
    }
}
