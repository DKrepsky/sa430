use sa430::{monitor::*, port::Port};

pub struct PrinterEventHandler<'a> {
    output: &'a mut dyn std::io::Write,
}

impl<'a> PrinterEventHandler<'a> {
    pub fn new(output: &'a mut dyn std::io::Write) -> PrinterEventHandler<'a> {
        PrinterEventHandler { output }
    }
}

impl<'a> EventHandler for PrinterEventHandler<'a> {
    fn handle(&mut self, event: &Event) {
        match event {
            Event::DeviceAdded(port) => print("Connected", &port, self.output),
            Event::DeviceRemoved(port) => print("Disconnected", &port, self.output),
        }
    }
}

/// Watches for SA430 connected/disconnected events using the provided monitor.
///
/// # Arguments
/// * `monitor` - The monitor to use to watch for events.
/// * `output` - The writer to output the events received.
///
/// # Note
/// The monitor will be started and will run indefinitely until the process is killed.
pub fn watch<'a>(monitor: &mut dyn Monitor<'a>, handler: &'a mut dyn EventHandler) -> std::io::Result<()> {
    monitor.subscribe(handler);
    monitor.start()
}

fn print(event_type: &str, port: &Port, output: &mut dyn std::io::Write) {
    writeln!(
        output,
        "{}: {:14} | {:16} | {:4}",
        event_type,
        port.name(),
        port.serial_number(),
        port.firmware_version()
    )
    .expect("Failed to write to output");
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMonitor<'a> {
        handlers: Vec<&'a mut dyn EventHandler>,
        started: u8,
    }

    impl<'a> MockMonitor<'a> {
        fn new() -> Self {
            MockMonitor {
                handlers: Vec::new(),
                started: 0,
            }
        }

        fn handlers(&mut self) -> &[&'a mut dyn EventHandler] {
            &self.handlers
        }

        fn started(&self) -> u8 {
            self.started
        }
    }

    impl<'a> Monitor<'a> for MockMonitor<'a> {
        fn start(&mut self) -> std::io::Result<()> {
            self.started += 1;
            for handler in self.handlers.iter_mut() {
                handler.handle(&Event::DeviceAdded(a_port()));
            }

            for handler in self.handlers.iter_mut() {
                handler.handle(&Event::DeviceRemoved(a_port()));
            }
            Ok(())
        }

        fn subscribe(&mut self, handler: &'a mut dyn EventHandler) {
            self.handlers.push(handler);
        }
    }

    fn a_port() -> Port {
        Port::new("/dev/ttyUSB1", "08FF41E50F8B3A34", "0104")
    }

    #[test]
    fn given_a_event_when_monitor_then_print_port_information() {
        let mut output = Vec::new();
        let mut handler = PrinterEventHandler::new(&mut output);
        let mut monitor = MockMonitor::new();

        watch(&mut monitor, &mut handler).expect("Failed to monitor");

        assert_eq!(monitor.started(), 1);
        assert_eq!(monitor.handlers().len(), 1);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Connected: /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n\
            Disconnected: /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n"
        );
    }
}
