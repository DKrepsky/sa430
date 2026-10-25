//! Waits for an SA430 to be connected, blinks its LED, and exits.
//!
//! ```bash
//! cargo run --example watch
//! ```
use std::ops::ControlFlow;

use sa430::{create_monitor, Event, Sa430};

fn main() -> sa430::Result<()> {
    println!("Waiting for an SA430 to be connected...");

    let mut connected = None;
    create_monitor()?.run(&mut |event: &Event| match event {
        Event::DeviceAdded(port) => {
            connected = Some(port.clone());
            ControlFlow::Break(())
        }
        Event::DeviceRemoved(_) => ControlFlow::Continue(()),
    })?;

    if let Some(port) = connected {
        println!("Connected on {}, blinking LED", port.name());
        Sa430::open(port.name())?.blink()?;
    }

    Ok(())
}
