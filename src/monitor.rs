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
use std::ops::ControlFlow;

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
    /// Blocks, passing every event to `handler`, until the handler returns [`ControlFlow::Break`] or an error occurs.
    fn run(&mut self, handler: &mut dyn EventHandler) -> Result<()>;
}
