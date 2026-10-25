use std::io;

use sa430::{Port, Scanner};

use super::{CliResult, UNKNOWN};

/// Scans for devices using the provided scanner and prints a table of the ports where they were found.
pub fn scan(scanner: &dyn Scanner, output: &mut dyn io::Write) -> CliResult {
    let ports = scanner.scan()?;
    writeln!(output, "port           | serial number    | version")?;
    writeln!(output, "---------------|------------------|--------")?;
    for port in ports {
        print_port_information(&port, output)?;
    }
    writeln!(output, "---------------|------------------|--------")?;
    Ok(())
}

fn print_port_information(port: &Port, output: &mut dyn io::Write) -> io::Result<()> {
    writeln!(
        output,
        "{:14} | {:16} | {:4}",
        port.name(),
        port.serial_number().unwrap_or(UNKNOWN),
        port.firmware_version().unwrap_or(UNKNOWN)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeScanner {
        ports: Vec<Port>,
    }

    impl Scanner for FakeScanner {
        fn scan(&self) -> sa430::Result<Vec<Port>> {
            Ok(self.ports.clone())
        }
    }

    #[test]
    fn given_a_device_is_connected_when_scan_then_print_port_information() {
        let mut output = Vec::new();
        let ports = vec![
            Port::new("/dev/ttyUSB1", Some("08FF41E50F8B3A34"), Some("0104")),
            Port::new("/dev/ttyUSB2", Some("08FF41E50F8B3A35"), Some("0104")),
            Port::new("COM3", Some("08FF41E50F8B3A36"), None),
        ];
        let scanner = FakeScanner { ports };

        scan(&scanner, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(
            output,
            "port           | serial number    | version\n\
         ---------------|------------------|--------\n\
         /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n\
         /dev/ttyUSB2   | 08FF41E50F8B3A35 | 0104\n\
         COM3           | 08FF41E50F8B3A36 | -   \n\
         ---------------|------------------|--------\n"
        );
    }

    #[test]
    fn given_no_device_is_connected_when_scan_then_print_no_port_information() {
        let mut output = Vec::new();
        let scanner = FakeScanner { ports: vec![] };

        scan(&scanner, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(
            output,
            "port           | serial number    | version\n\
         ---------------|------------------|--------\n\
         ---------------|------------------|--------\n"
        );
    }
}
