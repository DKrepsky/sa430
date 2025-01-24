use super::udev_utils::*;
use crate::monitor::*;

pub struct LinuxMonitor {
    handlers: Vec<Box<Handler>>,
}

impl LinuxMonitor {
    pub fn new() -> Self {
        LinuxMonitor { handlers: Vec::new() }
    }

    fn notify(&mut self, event: Event) {
        for handler in self.handlers.iter() {
            handler(event.clone());
        }
    }

    fn poll(&mut self, socket: &udev::MonitorSocket) {
        for event in socket.iter() {
            if is_sa430(&event.device()) {
                self.process(event)
            }
        }
    }

    fn process(&mut self, event: udev::Event) {
        let port = port_from_event(&event);

        match event.event_type() {
            udev::EventType::Add => self.notify(Event::DeviceAdded(port)),
            udev::EventType::Remove => self.notify(Event::DeviceRemoved(port)),
            _ => {}
        }
    }
}

impl Monitor for LinuxMonitor {
    fn subscribe(&mut self, handler: Box<Handler>) {
        self.handlers.push(handler);
    }

    fn start(&mut self) -> std::io::Result<()> {
        let socket = udev::MonitorBuilder::new()?.match_subsystem("tty")?.listen()?;

        loop {
            self.poll(&socket);
        }
    }
}
