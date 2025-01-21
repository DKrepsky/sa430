use std::{cell::RefCell, io::Result, io::Write, rc::Weak};

use sa430::{device::Device, monitor::*};

/// Watches for connected and disconnected devices using the provided monitor and outputs the device information using the provided writer.
///
/// # Arguments
/// * `monitor` - The monitor to use to watch for devices.
/// * `output` - The writer to output the devices found.
///
/// # Note
/// The monitor will be started and will run indefinitely until the process is killed.
pub fn watch(monitor: &mut dyn Monitor, output: Weak<RefCell<dyn Write>>) -> Result<()> {
    monitor.subscribe(handler_factory(output));
    monitor.start()
}

fn print(event_type: &str, device: &Device, output: &mut dyn Write) {
    writeln!(
        output,
        "{}: {:14} | {:16} | {:4}",
        event_type,
        device.port(),
        device.serial_number(),
        device.firmware_version()
    )
    .expect("Failed to write to output");
}

fn handler_factory(output: Weak<RefCell<dyn Write>>) -> Box<Handler> {
    Box::new(move |event: Event| match event {
        Event::DeviceAdded(device) => {
            if let Some(output) = output.upgrade() {
                let mut output = output.borrow_mut();
                print("Connected", &device, &mut *output);
            }
        }
        Event::DeviceRemoved(device) => {
            if let Some(output) = output.upgrade() {
                let mut output = output.borrow_mut();
                print("Disconnected", &device, &mut *output);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{ops::Deref, rc::Rc};

    use super::*;

    struct MockMonitor {
        handlers: Vec<Box<Handler>>,
        started: u8,
    }

    impl MockMonitor {
        fn new() -> Self {
            MockMonitor {
                handlers: Vec::new(),
                started: 0,
            }
        }

        fn handlers(&self) -> &Vec<Box<Handler>> {
            &self.handlers
        }

        fn started(&self) -> u8 {
            self.started
        }

        fn a_device(&self) -> Device {
            Device::new("/dev/ttyUSB1", "08FF41E50F8B3A34", "0104")
        }
    }

    impl Monitor for MockMonitor {
        fn subscribe(&mut self, handler: Box<Handler>) {
            self.handlers.push(handler);
        }

        fn start(&mut self) -> Result<()> {
            self.started += 1;
            for handler in self.handlers.iter() {
                handler(Event::DeviceAdded(self.a_device()));
            }

            for handler in self.handlers.iter() {
                handler(Event::DeviceRemoved(self.a_device()));
            }
            Ok(())
        }
    }

    struct VecWriter {
        inner: Rc<RefCell<Vec<u8>>>,
    }

    impl VecWriter {
        fn new(inner: Rc<RefCell<Vec<u8>>>) -> Self {
            VecWriter { inner }
        }
    }

    impl Write for VecWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> Result<()> {
            self.inner.borrow_mut().flush()
        }
    }

    #[test]
    fn given_a_event_when_monitor_then_print_device_information() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let mut monitor = MockMonitor::new();
        let writer = VecWriter::new(output.clone());
        let writer_ref: Rc<RefCell<dyn Write>> = Rc::new(RefCell::new(writer));

        watch(&mut monitor, Rc::downgrade(&writer_ref)).expect("Failed to monitor");

        let written = String::from_utf8(output.borrow().deref().clone()).unwrap();

        assert_eq!(monitor.started(), 1);
        assert_eq!(monitor.handlers().len(), 1);
        assert_eq!(
            written,
            "Connected: /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n\
            Disconnected: /dev/ttyUSB1   | 08FF41E50F8B3A34 | 0104\n"
        );
    }
}
