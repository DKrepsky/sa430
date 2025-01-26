use super::udev_utils::*;
use crate::monitor::*;

pub struct LinuxMonitor<'a> {
    handlers: Vec<&'a mut dyn EventHandler>,
}

impl LinuxMonitor<'_> {
    pub fn new<'a>() -> LinuxMonitor<'a> {
        LinuxMonitor { handlers: Vec::new() }
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
            udev::EventType::Add => self.notify(&Event::DeviceAdded(port)),
            udev::EventType::Remove => self.notify(&Event::DeviceRemoved(port)),
            _ => {}
        }
    }

    fn notify(&mut self, event: &Event) {
        for handler in self.handlers.iter_mut() {
            handler.handle(event);
        }
    }
}

impl<'a> Monitor<'a> for LinuxMonitor<'a> {
    fn subscribe(&mut self, handler: &'a mut dyn EventHandler) {
        self.handlers.push(handler);
    }

    fn start(&mut self) -> std::io::Result<()> {
        let socket = udev::MonitorBuilder::new()?.match_subsystem("tty")?.listen()?;

        loop {
            self.poll(&socket);
        }
    }
}
