# SA430 Sub-1 GHz RF Spectrum Analyzer Library for Rust

[![CI](https://github.com/DKrepsky/sa430/actions/workflows/ci.yml/badge.svg)](https://github.com/DKrepsky/sa430/actions/workflows/ci.yml)
![Codecov](https://img.shields.io/codecov/c/github/DKrepsky/sa430)
[![Crates.io](https://img.shields.io/crates/v/sa430.svg)](https://crates.io/crates/sa430)
![Crates.io Total Downloads](https://img.shields.io/crates/d/sa430)
![GitHub License](https://img.shields.io/github/license/DKrepsky/sa430)

## Overview

The **SA430** Rust library provides tools to interact with Texas Instruments SA430 Sub-1 GHz RF spectrum analyzers. It enables scanning, analyzing, and visualizing RF spectrum data. You can use it as a standalone CLI tool or integrate it into your Rust projects.

Device communication and scanning work on Linux, Windows and macOS. On Linux, `libudev` is required. Monitoring for
connect/disconnect events (`watch`) is currently Linux only.


## Installation

### CLI Application

For Linux systems, ensure the required dependency is installed:
```bash
sudo apt install libudev-dev
```

Then, install the CLI tool using Cargo:

```bash
cargo install sa430
```

### Rust Library
To use this library in your Rust project, add it as a dependency:

```bash
cargo add sa430
```

## Usage

### CLI Commands

`scan`: Lists all connected SA430 devices:

```bash
sa430 scan
```

`watch`: Monitors device connection and disconnection events:

```bash
sa430 watch
```

`info`: Read device basic information:

```bash
sa430 info <PORT>
```

`blink`: Blink the led.

```bash
sa430 blink <PORT>
```

`reboot`: Perform a hardware reset.

```bash
sa430 reboot <PORT>
```

`capture`: Measure the spectrum and print it as CSV (`frequency_mhz,power_dbm`). Frequencies are in MHz and must be
within 300-348, 389-464 or 779-928 MHz (start and stop in the same range); the step goes from 0.001 to 75 MHz. The
reference level (dBm) is one of -35 (default), -40, -45, -50, -55, -60, -65 or -70.

```bash
sa430 capture <PORT> --fstart 433 --fstop 434 --fstep 0.1 [--rlevel -35]
```

### Library Integration

Here’s an example of integrating the library into a Rust project:

```rust
use sa430::{create_scanner, Sa430};

fn main() -> sa430::Result<()> {
    for port in create_scanner().scan()? {
        let mut device = Sa430::open(port.name())?;
        println!("Found SA430 at {}: {}", port.name(), device.idn()?);
    }

    Ok(())
}
```

All fallible functions return `sa430::Result<T>`; match on `sa430::Error` to tell apart I/O errors (including
timeouts), corrupted frames, errors reported by the device and unsupported operations.

A `Sa430` is `Send` (every `Channel` must be), so it can be moved to a worker thread.

More examples can be found in the [examples](examples/) folder, such as scanning for devices and waiting for a device
to be connected:

```bash
cargo run --example scan
cargo run --example watch
```

### Testing code that uses this library

Enable the `test-utils` feature in your `dev-dependencies` to get `sa430::test_utils::MockChannel` and canned device
responses, so you can test without hardware.

## Troubleshooting

### Common Issues

- libudev not found: Ensure the libudev-dev package is installed on Linux:
```bash
sudo apt install libudev-dev
```
- No ports detected: Confirm that your SA430 device is connected and powered on. Use `sa430 scan` to verify.

- Permission error when opening the serial port: make sure your user has the `dialout` group.
```bash
# Logout/restart required after this command
sudo usermod -aG dialout $USER
```

## License
This library is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

