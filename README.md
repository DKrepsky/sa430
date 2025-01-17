# SA430 Sub-1 GHz RF Spectrum Analyzer Tool Rust Library

[![CI](https://github.com/DKrepsky/sa430/actions/workflows/ci.yml/badge.svg)](https://github.com/DKrepsky/sa430/actions/workflows/ci.yml)
![Codecov](https://img.shields.io/codecov/c/github/DKrepsky/sa430)
[![Crates.io](https://img.shields.io/crates/v/sa430.svg)](https://crates.io/crates/sa430)
![Crates.io Total Downloads](https://img.shields.io/crates/d/sa430)
![GitHub License](https://img.shields.io/github/license/DKrepsky/sa430)


## Overview

The SA430 Rust library provides a set of tools for interacting with Texas Instruments SA430 Sub-1 GHz RF spectrum analyzers. It allows users to scan, analyze, and visualize RF spectrum data. This library is designed to be used in both command-line applications and as a dependency in other Rust projects.

## Supported Operational Systems

Most functionalities works on all platforms, except the scan, which at the time only support unix based systems that are libudev compatible.

## Installation

### Standalone CLI Application

Install the dependencies (Linux Only):
```bash
sudo apt install libudev-dev
```

You can use cargo to install the application:

```bash
cargo install sa430
```

Then run the [commands](#commands), for example:
```bash
sa430 scan
```

### Library in Other Rust Projects

Add the project dependency:
```bash
cargo add sa430
```

Checkout the [examples](examples/) folder for usage.


## CLI Usage

- `scan`: List all connected spectrum analyzers.
- `watch`: Watch for device connected/disconnected events.

