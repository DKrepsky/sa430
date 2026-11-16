//! Detecting SA430 connect/disconnect events.
//!
//! A [`Monitor`] watches for SA430 devices being plugged in or removed and passes each [`Event`] to an
//! [`EventHandler`]. The handler decides whether monitoring continues by returning [`ControlFlow`]. Any
//! `FnMut(&Event) -> ControlFlow<()>` closure is an `EventHandler`.
//!
//! Use [`create_monitor`](crate::create_monitor) to get the implementation for the current OS (Linux only for now).
//! Implement `Monitor` yourself only to support another operating system.
//!
//! # Examples
//!
//! ```no_run
//! use std::ops::ControlFlow;
//! use sa430::monitor::Event;
//!
//! let mut monitor = sa430::create_monitor()?;
//! monitor.run(&mut |event: &Event| {
//!     match event {
//!         Event::DeviceAdded(port) => println!("Connected: {}", port.name()),
//!         Event::DeviceRemoved(port) => println!("Disconnected: {}", port.name()),
//!     }
//!     ControlFlow::Continue(())
//! })?;
//! # Ok::<(), sa430::Error>(())
//! ```
//!
//! # Stopping from another thread
//!
//! `run` blocks, so a caller on another thread cannot end it with the handler alone. Take a
//! [`MonitorStopHandle`] before calling `run`, move the monitor to a worker thread, and call
//! [`MonitorStopHandle::stop`] from anywhere to make `run` return `Ok(())`.
//!
//! ```no_run
//! let mut monitor = sa430::create_monitor()?;
//! let stop = monitor.stop_handle().expect("this monitor can be stopped");
//! let worker = std::thread::spawn(move || monitor.run(&mut |_: &sa430::Event| std::ops::ControlFlow::Continue(())));
//!
//! // ... later, from any thread:
//! stop.stop();
//! worker.join().expect("monitor thread panicked")?;
//! # Ok::<(), sa430::Error>(())
//! ```
use std::fmt;
use std::ops::ControlFlow;
use std::sync::Arc;

use crate::error::Result;
use crate::port::Port;

/// An SA430 connection event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A device was connected to the port.
    DeviceAdded(Port),
    /// A device was disconnected from the port.
    DeviceRemoved(Port),
}

/// Receives events from a [`Monitor`].
pub trait EventHandler {
    /// Called for every event. Return [`ControlFlow::Break`] to stop the monitor.
    fn handle(&mut self, event: &Event) -> ControlFlow<()>;
}

impl<F: FnMut(&Event) -> ControlFlow<()>> EventHandler for F {
    fn handle(&mut self, event: &Event) -> ControlFlow<()> {
        self(event)
    }
}

/// Watches for SA430 connect/disconnect events.
pub trait Monitor {
    /// Blocks, passing every event to `handler`, until the handler returns [`ControlFlow::Break`], [`stop`] is called
    /// on a handle from [`stop_handle`](Monitor::stop_handle), or an error occurs.
    ///
    /// [`stop`]: MonitorStopHandle::stop
    fn run(&mut self, handler: &mut dyn EventHandler) -> Result<()>;

    /// Returns a handle that stops [`run`](Monitor::run) from another thread, or `None` if this monitor cannot be
    /// stopped that way.
    ///
    /// The handle stays valid after `run` returns. Once stopped, the monitor stays stopped: later calls to `run`
    /// return at once. Implementations that return `Some` must make `stop` wake a blocked `run`.
    ///
    /// This is a provided method so that adding it did not break existing implementations of `Monitor`.
    fn stop_handle(&self) -> Option<MonitorStopHandle> {
        None
    }
}

/// Stops a [`Monitor`] from any thread. Obtain one with [`Monitor::stop_handle`].
///
/// Cloning the handle is cheap and every clone stops the same monitor. Calling [`stop`](Self::stop) more than once is
/// harmless.
#[derive(Clone)]
pub struct MonitorStopHandle {
    stop: Arc<dyn Fn() + Send + Sync>,
}

impl MonitorStopHandle {
    /// Creates a handle that runs `stop` when [`stop`](Self::stop) is called. Used by [`Monitor`] implementations
    /// to wire their own wake-up mechanism.
    pub fn new(stop: impl Fn() + Send + Sync + 'static) -> Self {
        MonitorStopHandle { stop: Arc::new(stop) }
    }

    /// Asks the monitor to stop. A `run` that is blocked returns `Ok(())` as soon as possible. Does nothing if the
    /// monitor has already been stopped.
    pub fn stop(&self) {
        (self.stop)()
    }
}

impl fmt::Debug for MonitorStopHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MonitorStopHandle").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct NotStoppable;

    impl Monitor for NotStoppable {
        fn run(&mut self, _handler: &mut dyn EventHandler) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn given_a_monitor_without_stop_support_when_stop_handle_then_none() {
        assert!(NotStoppable.stop_handle().is_none());
    }

    #[test]
    fn given_a_stop_handle_when_stop_called_on_a_clone_then_the_closure_runs_each_time() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let handle = MonitorStopHandle::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });

        handle.clone().stop();
        handle.stop();

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn given_a_stop_handle_when_checked_then_it_is_send_sync_and_clone() {
        fn assert_send_sync_clone<T: Send + Sync + Clone>() {}
        assert_send_sync_clone::<MonitorStopHandle>();
    }
}
