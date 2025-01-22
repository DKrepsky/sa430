# SA430 Sub-1 GHz RF Spectrum Analyzer Library for Rust

[![CI](https://github.com/DKrepsky/sa430/actions/workflows/ci.yml/badge.svg)](https://github.com/DKrepsky/sa430/actions/workflows/ci.yml)
![Codecov](https://img.shields.io/codecov/c/github/DKrepsky/sa430)
[![Crates.io](https://img.shields.io/crates/v/sa430.svg)](https://crates.io/crates/sa430)
![Crates.io Total Downloads](https://img.shields.io/crates/d/sa430)
![GitHub License](https://img.shields.io/github/license/DKrepsky/sa430)

## Overview

The **SA430** Rust library provides tools to interact with Texas Instruments SA430 Sub-1 GHz RF spectrum analyzers. It enables scanning, analyzing, and visualizing RF spectrum data. You can use it as a standalone CLI tool or integrate it into your Rust projects.

This library is primarily Unix-compatible, requiring `libudev` for scanning operations. Other features are cross-platform.


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

### Library Integration

Here’s an example of integrating the library into a Rust project:

```
use sa430::{create_scanner, Scanner, Port};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = create_scanner();
    let ports = scanner.scan();

    for port in ports {
        println!("Found SA430 at: {}", port.name());
    }

    Ok(())
}
```

More examples can be found in the [examples](examples/) folder, like monitoring for port events and taking measurements.

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

