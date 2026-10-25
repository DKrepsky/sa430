use std::io;
use std::ops::ControlFlow;
use std::os::fd::AsRawFd;

use super::udev_utils::*;
use crate::error::Result;
use crate::monitor::*;

/// Monitor based on udev netlink events.
#[derive(Debug, Default)]
pub struct LinuxMonitor;

impl LinuxMonitor {
    pub fn new() -> Self {
        LinuxMonitor
    }
}

impl Monitor for LinuxMonitor {
    fn run(&mut self, handler: &mut dyn EventHandler) -> Result<()> {
        let socket = udev::MonitorBuilder::new()?.match_subsystem("tty")?.listen()?;

        loop {
            // The udev socket is non-blocking: sleep until it has data instead of spinning.
            wait_readable(&socket)?;

            for event in socket.iter() {
                if !is_sa430(&event.device()) {
                    continue;
                }

                let Some(port) = port_from_device(&event.device()) else {
                    continue;
                };

                let event = match event.event_type() {
                    udev::EventType::Add => Event::DeviceAdded(port),
                    udev::EventType::Remove => Event::DeviceRemoved(port),
                    _ => continue,
                };

                if let ControlFlow::Break(()) = handler.handle(&event) {
                    return Ok(());
                }
            }
        }
    }
}

/// Blocks until `fd` is readable.
fn wait_readable(fd: &impl AsRawFd) -> io::Result<()> {
    let mut pollfd = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    loop {
        // SAFETY: `pollfd` is a valid, initialized array of one element that outlives the call.
        let result = unsafe { libc::poll(&mut pollfd, 1, -1) };
        if result >= 0 {
            return Ok(());
        }

        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;

    use super::*;

    #[test]
    fn given_a_readable_fd_when_wait_readable_then_return() {
        let (reader, writer) = UnixStream::pair().unwrap();
        std::io::Write::write_all(&mut &writer, b"x").unwrap();
        wait_readable(&reader).unwrap();
    }
}
