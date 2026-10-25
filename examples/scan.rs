//! Lists connected SA430 devices and prints their identification.
//!
//! ```bash
//! cargo run --example scan
//! ```
use sa430::{create_scanner, Sa430};

fn main() -> sa430::Result<()> {
    let ports = create_scanner().scan()?;
    if ports.is_empty() {
        println!("No SA430 found");
    }

    for port in ports {
        let mut device = Sa430::open(port.name())?;
        println!(
            "{}: {} (serial {})",
            port.name(),
            device.idn()?,
            device.serial_number()?
        );
    }

    Ok(())
}
