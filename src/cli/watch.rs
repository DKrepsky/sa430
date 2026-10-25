use std::io;
use std::ops::ControlFlow;

use sa430::{Event, EventHandler, Monitor, Port};

use super::{CliResult, UNKNOWN};

/// Prints every event to `output`. Stops the monitor if the output can no longer be written.
pub struct PrinterEventHandler<'a> {
    output: &'a mut dyn io::Write,
    error: Option<io::Error>,
}

impl<'a> PrinterEventHandler<'a> {
    pub fn new(output: &'a mut dyn io::Write) -> Self {
        PrinterEventHandler { output, error: None }
    }
}

impl EventHandler for PrinterEventHandler<'_> {
    fn handle(&mut self, event: &Event) -> ControlFlow<()> {
        let result = match event {
            Event::DeviceAdded(port) => print("Connected", port, self.output),
            Event::DeviceRemoved(port) => print("Disconnected", port, self.output),
        };

        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => {
                self.error = Some(error);
                ControlFlow::Break(())
            }
        }
    }
}

/// Watches for SA430 connected/disconnected events and prints them to `output`.
///
/// Runs until the process is killed, the monitor fails or `output` can no longer be written.
pub fn watch(monitor: &mut dyn Monitor, output: &mut dyn io::Write) -> CliResult {
    let mut handler = PrinterEventHandler::new(output);
    monitor.run(&mut handler)?;
    match handler.error {
        Some(error) => Err(error.into()),
        None => Ok(()),
    }
}

fn print(event_type: &str, port: &Port, output: &mut dyn io::Write) -> io::Result<()> {
    writeln!(
        output,
        "{}: {:14} | {:16} | {:4}",
        event_type,
        port.name(),
        port.serial_number().unwrap_or(UNKNOWN),
        port.firmware_version().unwrap_or(UNKNOWN)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Replays a fixed list of events.
    struct MockMonitor {
        events: Vec<Event>,
    }

    impl Monitor for MockMonitor {
        fn run(&mut self, handler: &mut dyn EventHandler) -> sa430::Result<()> {
            for event in &self.events {
                if handler.handle(event).is_break() {
                    break;
                }
            }
            Ok(())
        }
    }

    /// A writer that always fails.
    struct BrokenPipe;

    impl io::Write for BrokenPipe {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn a_port() -> Port {
        Port::new("/dev/ttyUSB1", Some("08FF41E50F8B3A34"), Some("0104"))
    }

    fn a_monitor() -> MockMonitor {
        MockMonitor {
            events: vec![Event::DeviceAdded(a_port()), Event::DeviceRemoved(a_port())],
        }
    }

    #[test]
    fn given_a_event_when_monitor_then_print_port_information() {
        let mut output = Vec::new();

        watch(&mut a_monitor(), &mut output).expect("Failed to monitor");

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Connected: /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n\
            Disconnected: /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n"
        );
    }

    #[test]
    fn given_output_is_broken_when_monitor_then_stop_and_return_error() {
        let result = watch(&mut a_monitor(), &mut BrokenPipe);
        assert!(result.is_err());
    }
}
