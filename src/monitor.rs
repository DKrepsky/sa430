//! Provides functionality to detect USB connect/disconnect events.
//!
//! The module defines an `Event` enum to represent device addition and removal events, and a `Monitor` trait
//! that allows subscribing to these events and starting the monitoring process.
//!
//! The `Event` enum includes:
//! - `DeviceAdded(Device)`: Indicates that a new device has been connected.
//! - `DeviceRemoved(Device)`: Indicates that a device has been disconnected.
//!
//! To get a monitor instance, the user must call the `sa430::create_monitor()` function, which provides an OS-specific
//! implementation. Users should only implement the `Monitor` trait if they want to support operating systems other than
//! the currently available (Linux).
//!
//! # Note
//! When start is called, the monitor will run indefinitely until the process/thread is killed.
//!
//! # Examples
//!
//! ```ignore
//! use sa430::create_monitor;
//! use sa430::device::Device;
//! use sa430::monitor::{Monitor, Event};
//!
//! // Create a monitor and subscribe to device events.
//! fn main() -> std::io::Result<()> {
//!     let mut monitor = create_monitor();
//!     monitor.subscribe(Box::new(|event| match event {
//!         Event::DeviceAdded(device) => println!("Device added: {:?}", device),
//!         Event::DeviceRemoved(device) => println!("Device removed: {:?}", device),
//!     }));
//!     monitor.start()
//! }
//! ```

use super::device::Device;

/// Represents an event that can occur during device monitoring.
///
/// The `Event` enum includes:
/// - `DeviceAdded(Device)`: Indicates that a new device has been connected.
/// - `DeviceRemoved(Device)`: Indicates that a device has been disconnected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    DeviceAdded(Device),
    DeviceRemoved(Device),
}

/// A handler is a function that processes an event.
pub type Handler = dyn Fn(Event);

/// A monitor is responsible for monitoring devices connected to the computer.
pub trait Monitor {
    /// Subscribes to device events.
    fn subscribe(&mut self, handler: Box<Handler>);

    /// Starts the monitor.
    fn start(&mut self) -> std::io::Result<()>;
}
