//! Provides functionality to detect USB connect/disconnect events.
//!
//! The module defines an `Event` enum to represent device addition and removal events, and a `Monitor` trait
//! that allows an `EventHandler` to subscribe to these events and starting the monitoring process.
//!
//! The `Event` enum includes:
//! - `DeviceAdded(Port)`: Indicates that a new device has been connected to the `Port`.
//! - `DeviceRemoved(Port)`: Indicates that a device has been disconnected from `Port`.
//!
//! The `EventHandler` trait defines the behavior for handling these events, which includes:
//! - `handle(&mut self, event: &Event)`: This method is called when an event occurs.
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
//! use sa430::port::Port;
//! use sa430::monitor::{Monitor, Event};
//!
//! // Create a monitor and subscribe to usb events.
//! struct SomeEventHandler;
//!
//! impl sa430::monitor::EventHandler for SomeEventHandler {
//!   fn handle(&mut self, event: &Event) {
//!     match event {
//!       Event::DeviceAdded(port) => println!("Device added: {:?}", port),
//!       Event::DeviceRemoved(port) => println!("Device removed: {:?}", port),
//!     }
//!   }
//! }
//! let mut monitor = create_monitor();
//! let mut handler = SomeEventHandler{};
//! monitor.subscribe(&mut handler);
//! monitor.start()
//! ```

use super::port::Port;

/// Represents an event that can occur during device monitoring.
///
/// The `Event` enum includes:
/// - `DeviceAdded(Device)`: Indicates that a new device has been connected.
/// - `DeviceRemoved(Device)`: Indicates that a device has been disconnected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    DeviceAdded(Port),
    DeviceRemoved(Port),
}

/// Defines the behavior for handling events.
///
/// To handle events, implement the `EventHandler` trait and override the `handle` method, then subscribe to the
/// monitor.
pub trait EventHandler {
    fn handle(&mut self, event: &Event);
}

/// A monitor is responsible for monitoring devices connected to the computer.
pub trait Monitor<'a> {
    /// Subscribes to usb events.
    fn subscribe(&mut self, handler: &'a mut dyn EventHandler);

    /// Starts the monitor.
    fn start(&mut self) -> std::io::Result<()>;
}
