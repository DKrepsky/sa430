use std::io;
use std::ops::ControlFlow;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;

use super::udev_utils::*;
use crate::error::Result;
use crate::monitor::*;

/// Monitor based on udev netlink events.
///
/// Owns an eventfd that [`MonitorStopHandle::stop`] writes to, so a blocked `run` wakes up and returns.
#[derive(Debug)]
pub struct LinuxMonitor {
    stop_fd: Arc<OwnedFd>,
}

impl LinuxMonitor {
    pub fn new() -> io::Result<Self> {
        // SAFETY: `eventfd` takes no pointers; a negative return is checked below.
        let fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: `fd` is a new descriptor that nothing else owns, so the `OwnedFd` is its only closer.
        let stop_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        Ok(LinuxMonitor {
            stop_fd: Arc::new(stop_fd),
        })
    }
}

impl Monitor for LinuxMonitor {
    fn run(&mut self, handler: &mut dyn EventHandler) -> Result<()> {
        let socket = udev::MonitorBuilder::new()?.match_subsystem("tty")?.listen()?;

        loop {
            // The udev socket is non-blocking: sleep until it has data or a stop is requested, instead of spinning.
            let wakeup = wait_readable(&socket, self.stop_fd.as_ref())?;
            if wakeup.stop {
                return Ok(());
            }
            if !wakeup.device {
                continue;
            }

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

    fn stop_handle(&self) -> Option<MonitorStopHandle> {
        let stop_fd = Arc::clone(&self.stop_fd);
        Some(MonitorStopHandle::new(move || signal(&stop_fd)))
    }
}

/// Makes the stop eventfd readable. A failed write is ignored: the only failure is a counter overflow, which needs
/// about 2^64 calls, and there is no caller that could act on it.
fn signal(stop_fd: &OwnedFd) {
    let one: u64 = 1;
    // SAFETY: writes 8 bytes from a live `u64` to an eventfd that stays open while `stop_fd` is borrowed.
    let _ = unsafe {
        libc::write(
            stop_fd.as_raw_fd(),
            (&one as *const u64).cast(),
            std::mem::size_of::<u64>(),
        )
    };
}

/// Which of the two descriptors passed to [`wait_readable`] became ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Wakeup {
    device: bool,
    stop: bool,
}

/// `revents` bits that mean the udev socket is broken rather than readable. A broken socket keeps `poll` returning
/// at once with nothing to read, so `run` would spin on it.
const DEVICE_ERROR_EVENTS: libc::c_short = libc::POLLERR | libc::POLLNVAL | libc::POLLHUP;

/// Fails if the device fd reported an error condition in `revents`.
fn check_device_revents(revents: libc::c_short) -> io::Result<()> {
    if revents & DEVICE_ERROR_EVENTS == 0 {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "udev monitor socket failed (poll revents {:#06x})",
        revents as u16
    )))
}

/// Blocks until `device` or `stop` is readable, and reports which one. Fails if `device` is in an error state.
fn wait_readable(device: &impl AsRawFd, stop: &impl AsRawFd) -> io::Result<Wakeup> {
    let mut pollfds = [
        libc::pollfd {
            fd: device.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: stop.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
    ];

    loop {
        // SAFETY: `pollfds` is a valid, initialized array of two elements that outlives the call.
        let result = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as libc::nfds_t, -1) };
        if result >= 0 {
            check_device_revents(pollfds[0].revents)?;
            return Ok(Wakeup {
                device: pollfds[0].revents != 0,
                stop: pollfds[1].revents != 0,
            });
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
    use std::time::{Duration, Instant};

    use super::*;

    /// Upper bound for "promptly": a stop must end the wait well before the human notices a delay.
    const PROMPT: Duration = Duration::from_secs(1);

    /// A device fd that never has events, so only the stop fd can wake the wait.
    fn idle_device() -> (UnixStream, UnixStream) {
        UnixStream::pair().unwrap()
    }

    fn stop_fd_of(monitor: &LinuxMonitor) -> Arc<OwnedFd> {
        Arc::clone(&monitor.stop_fd)
    }

    fn stop_handle_of(monitor: &LinuxMonitor) -> MonitorStopHandle {
        monitor.stop_handle().expect("LinuxMonitor is stoppable")
    }

    #[test]
    fn given_a_readable_device_when_wait_readable_then_report_device_only() {
        let monitor = LinuxMonitor::new().unwrap();
        let (reader, writer) = UnixStream::pair().unwrap();
        std::io::Write::write_all(&mut &writer, b"x").unwrap();

        let wakeup = wait_readable(&reader, stop_fd_of(&monitor).as_ref()).unwrap();

        assert_eq!(
            wakeup,
            Wakeup {
                device: true,
                stop: false
            }
        );
    }

    #[test]
    fn given_a_stop_from_another_thread_when_wait_readable_then_return_stop_promptly() {
        let monitor = LinuxMonitor::new().unwrap();
        let stop = stop_handle_of(&monitor);
        let (reader, _writer) = idle_device();

        let started = Instant::now();
        let stopper = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            stop.stop();
        });
        let wakeup = wait_readable(&reader, stop_fd_of(&monitor).as_ref()).unwrap();
        stopper.join().unwrap();

        assert!(started.elapsed() < PROMPT, "wait took {:?}", started.elapsed());
        assert_eq!(
            wakeup,
            Wakeup {
                device: false,
                stop: true
            }
        );
    }

    #[test]
    fn given_stop_called_twice_when_wait_readable_then_return_stop_immediately() {
        let monitor = LinuxMonitor::new().unwrap();
        let stop = stop_handle_of(&monitor);
        let (reader, _writer) = idle_device();

        stop.stop();
        stop.clone().stop();
        let wakeup = wait_readable(&reader, stop_fd_of(&monitor).as_ref()).unwrap();

        assert!(wakeup.stop);
    }

    #[test]
    fn given_stop_before_run_when_run_then_return_ok_without_calling_handler() {
        let mut monitor = LinuxMonitor::new().unwrap();
        stop_handle_of(&monitor).stop();

        let mut calls = 0;
        let started = Instant::now();
        let result = monitor.run(&mut |_: &Event| {
            calls += 1;
            ControlFlow::Continue(())
        });

        assert!(result.is_ok());
        assert!(started.elapsed() < PROMPT);
        assert_eq!(calls, 0);
    }

    #[test]
    fn given_run_blocked_when_stop_from_another_thread_then_run_returns_ok_promptly() {
        let mut monitor = LinuxMonitor::new().unwrap();
        let stop = stop_handle_of(&monitor);
        let (sender, receiver) = std::sync::mpsc::channel();

        let worker = std::thread::spawn(move || {
            let result = monitor.run(&mut |_: &Event| ControlFlow::Continue(()));
            sender.send(result).unwrap();
        });

        std::thread::sleep(Duration::from_millis(50));
        let started = Instant::now();
        stop.stop();
        let result = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("run did not return after stop");
        worker.join().unwrap();

        assert!(result.is_ok());
        assert!(started.elapsed() < PROMPT, "run took {:?} to return", started.elapsed());
    }

    #[test]
    fn given_device_error_flags_when_check_device_revents_then_return_err() {
        for revents in [
            libc::POLLERR,
            libc::POLLNVAL,
            libc::POLLHUP,
            libc::POLLIN | libc::POLLHUP,
            libc::POLLIN | libc::POLLERR,
        ] {
            assert!(check_device_revents(revents).is_err(), "revents {revents:#x}");
        }
    }

    #[test]
    fn given_no_error_flags_when_check_device_revents_then_return_ok() {
        for revents in [0, libc::POLLIN, libc::POLLOUT] {
            assert!(check_device_revents(revents).is_ok(), "revents {revents:#x}");
        }
    }

    #[test]
    fn given_peer_closed_when_wait_readable_then_return_err_instead_of_spinning() {
        let monitor = LinuxMonitor::new().unwrap();
        let (reader, writer) = UnixStream::pair().unwrap();
        drop(writer);

        let started = Instant::now();
        let result = wait_readable(&reader, stop_fd_of(&monitor).as_ref());

        assert!(result.is_err(), "expected an error, got {result:?}");
        assert!(started.elapsed() < PROMPT);
    }

    #[test]
    fn given_monitor_when_stop_handle_then_it_is_some() {
        let monitor = LinuxMonitor::new().unwrap();
        assert!(monitor.stop_handle().is_some());
    }
}
