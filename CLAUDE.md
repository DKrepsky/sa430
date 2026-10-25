# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Rust library + CLI (`sa430` binary) for the Texas Instruments SA430 Sub-1 GHz RF spectrum analyzer. The device appears as a USB serial port (VID `0x2047`, PID `0x0005`). `docs/protocol.md` is the reverse-engineered reference for the serial protocol (frame layout, command codes, error codes, CRC, flash layout). Read it before touching `frame.rs`/`protocol.rs`/`device.rs` or adding a command.

## Commands

Linux builds require `libudev-dev` (`sudo apt install libudev-dev`).

```bash
cargo build
cargo test                                     # unit + integration + doc tests
cargo test <test_name_substring>               # single test, e.g. cargo test given_a_device_when_info
cargo test --test integration_tests            # only tests/integration_tests.rs (runs the CLI binary)
cargo clippy --all-targets --all-features -- -D warnings   # lint (CI fails on warnings)
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps # doc check (CI)
cargo fmt -- --check                           # format check (CI); max_width = 120 via rustfmt.toml
cargo llvm-cov --lcov --output-path lcov.info  # coverage
cargo run -- <scan|watch|info|blink|reboot|capture> [PORT] ...
cargo run --example scan                       # examples/ needs real hardware
```

CI runs tests on Linux, Windows and macOS, so non-Linux code paths must compile. `cargo check --target x86_64-pc-windows-gnu --all-targets` is a quick local check.

## Architecture

Layered, from hardware up:

- **`channel.rs`**: `Channel` is a marker trait with a blanket impl for every `Read + Write` type. `SerialPortChannel` opens the port with the device's fixed settings (926100 baud, 8N1, 5 s timeout) and clears stale input on open.
- **`frame.rs`**: `Frame` (magic `0x2A`, length, `Command`, ≤255 data bytes, big-endian CRC16 from `crc.rs`), plus the `Command`/`ErrorCode` enums. `Frame::with_data` is fallible and enforces the 255-byte limit, so `to_bytes` can't truncate.
- **`protocol.rs`** (crate-private): every command is answered with an **ACK frame** first, then (for commands with results) a **response frame**. `transact` validates both: an error frame becomes `Error::Device`, and a mismatched command becomes `Error::UnexpectedResponse`. `read_flash` splits reads into ≤255-byte chunks, rejects short responses and truncates padded ones.
- **`parser.rs`** (crate-private): `ByteArrayParser`, a bounds-checked cursor (big-endian by default, plus `take_u16_le` for the little-endian flash `ProgHeader`).
- **`device.rs`**: `Sa430`, the high-level API (`Sa430::open(port)` or `Sa430::new(Box<dyn Channel>)`). Calibration data is read from flash (a `ProgHeader`, then a 1671-byte block), decoded into `Calibration`, and cached.
- **`error.rs`**: a single `Error` enum (`#[non_exhaustive]`) and `Result<T>` alias, used by every public fallible function. Library code must not panic on device or OS input; return an `Error` instead.
- **Discovery**: `Scanner::scan() -> Result<Vec<Port>>`. `create_scanner()` returns the udev `LinuxScanner` on Linux (it also reports the firmware revision) and the cross-platform `SerialPortScanner` elsewhere. `Monitor::run(&mut dyn EventHandler)` blocks until the handler returns `ControlFlow::Break`. Closures implement `EventHandler`. `create_monitor()` returns `Error::Unsupported` off Linux. udev-specific code (property keys, `poll()` on the non-blocking udev socket) lives only in `src/linux/`.
- **CLI**: `main.rs` (clap derive) parses arguments, calls `run()`, prints errors as `error: …` and exits 1. Each subcommand lives in `src/cli/<cmd>.rs` as a function taking `&mut Sa430` (or `&dyn Scanner`/`&mut dyn Monitor`) and `&mut dyn io::Write`, returning `CliResult`. That keeps them unit-testable against a buffer. `capture` validates its arguments but is not implemented yet (issue #6).

Adding a device command generally means: add the `Command` variant in `frame.rs` (enum, `Display`, `From<u8>`) → method on `Sa430` using a `protocol` helper → `src/cli/<cmd>.rs` + subcommand in `main.rs` → README usage entry.

## Testing conventions

- Test names follow `given_..._when_..._then_...`.
- Hardware is faked with `sa430::test_utils` (compiled only under `cfg(test)` or the `test-utils` feature). The crate enables that feature for its own tests through a self dev-dependency in `Cargo.toml`, which is how the binary's CLI tests can use it. `MockChannel::add_response(&Frame)` queues frames, and reading past the queue returns a `TimedOut` I/O error like a real port. Queue the ACK before each response (`an_ack_response(cmd)`), or use `add_calibration_responses` for the full calibration read.
- Scanner and monitor have no hardware-free integration tests (see `NOTES.md`). Their pure logic (`scanner::to_sa430_port`, `linux::monitor::wait_readable`) is unit-tested.
