//! # Channel Module
//!
//! This module provides an abstraction for the communication link with the device. A [`Channel`] is anything that
//! can be read from and written to; it is implemented automatically for every `io::Read + io::Write` type, so tests can
//! use in-memory buffers. [`SerialPortChannel`] is the implementation used with real hardware.
//!
//! ## Serial port settings
//!
//! - Baud rate: 926100 bps.
//! - Data bits: 8.
//! - Stop bits: 1.
//! - Parity: none.
//! - Flow control: none. The device advertises hardware flow control, but does not use it (see `docs/protocol.md`).
//! - Read timeout: 5 seconds by default, changeable with [`SerialPortChannel::set_timeout`].
//!
//! ## Opening the port
//!
//! A device that has just enumerated (it was plugged in, or it was rebooted with
//! [`Sa430::reboot`](crate::device::Sa430::reboot)) is not ready to receive a command straight away: a frame written
//! within about 20 ms of opening the port is corrupted, and the device then stays one reply behind (see
//! `docs/protocol.md`). [`SerialPortChannel::new`] therefore waits [`DEFAULT_SETTLE_DELAY`] after opening, then
//! discards any stale input. Use [`SerialPortChannel::with_settle`] to pick another delay, for example
//! `Duration::ZERO` when the port is already known to be ready.
//!
//! ## Usage Example
//!
//! ```no_run
//! use sa430::channel::SerialPortChannel;
//! use sa430::device::Sa430;
//!
//! fn main() -> sa430::Result<()> {
//!     let channel = SerialPortChannel::new("/dev/ttyACM0")?;
//!     let mut device = Sa430::new(Box::new(channel));
//!     println!("{}", device.idn()?);
//!     Ok(())
//! }
//! ```
use std::{io, thread, time::Duration};

use serialport::{ClearBuffer, DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::error::Result;

const SERIAL_PORT_BAUD_RATE: u32 = 926100;
const SERIAL_PORT_STOP_BITS: StopBits = StopBits::One;
const SERIAL_PORT_DATA_BITS: DataBits = DataBits::Eight;
const SERIAL_PORT_PARITY: Parity = Parity::None;
const SERIAL_PORT_FLOW_CONTROL: FlowControl = FlowControl::None;
const SERIAL_PORT_TIMEOUT: Duration = Duration::from_secs(5);

/// Time [`SerialPortChannel::new`] waits after opening the port, before discarding stale input and returning.
///
/// Measured on a real SA430 after it enumerated: waits of 5 to 300 ms all worked, while a frame sent within about 20 ms
/// of opening was corrupted. 200 ms leaves a wide margin above the measured minimum.
pub const DEFAULT_SETTLE_DELAY: Duration = Duration::from_millis(200);

/// A bidirectional, blocking byte stream connected to a device.
///
/// Channels are `Send` so a [`Sa430`](crate::Sa430) can be moved to a worker thread.
pub trait Channel: io::Read + io::Write + Send {}

impl<T: io::Read + io::Write + Send + ?Sized> Channel for T {}

/// A [`Channel`] backed by a serial port configured for the SA430.
pub struct SerialPortChannel {
    port: Box<dyn SerialPort>,
}

impl SerialPortChannel {
    /// Opens `port_name` with the settings used by the SA430 hardware and waits [`DEFAULT_SETTLE_DELAY`].
    ///
    /// Any bytes already waiting in the input buffer (e.g. left over from an interrupted session) are discarded, so
    /// they are not mistaken for a response to the next command.
    pub fn new(port_name: &str) -> Result<Self> {
        Self::with_settle(port_name, DEFAULT_SETTLE_DELAY)
    }

    /// Opens `port_name` like [`SerialPortChannel::new`], but waits `settle` instead of [`DEFAULT_SETTLE_DELAY`].
    ///
    /// The wait happens after the port is opened and before the input is discarded. A zero duration skips the wait;
    /// use it only when the device is known to be ready, such as an already-running port.
    pub fn with_settle(port_name: &str, settle: Duration) -> Result<Self> {
        let port = serialport::new(port_name, SERIAL_PORT_BAUD_RATE)
            .stop_bits(SERIAL_PORT_STOP_BITS)
            .data_bits(SERIAL_PORT_DATA_BITS)
            .parity(SERIAL_PORT_PARITY)
            .flow_control(SERIAL_PORT_FLOW_CONTROL)
            .timeout(SERIAL_PORT_TIMEOUT)
            .open()?;

        Self::settled(port, settle)
    }

    /// Sets how long a read waits for data before failing with [`io::ErrorKind::TimedOut`].
    ///
    /// The timeout starts at 5 seconds. Applies to all later reads on this channel.
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        self.port.set_timeout(timeout)?;
        Ok(())
    }

    /// Returns the current read timeout (see [`SerialPortChannel::set_timeout`]).
    pub fn timeout(&self) -> Duration {
        self.port.timeout()
    }

    fn settled(port: Box<dyn SerialPort>, settle: Duration) -> Result<Self> {
        thread::sleep(settle);
        port.clear(ClearBuffer::Input)?;

        Ok(SerialPortChannel { port })
    }
}

impl io::Read for SerialPortChannel {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.port.read(buf)
    }
}

impl io::Write for SerialPortChannel {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.port.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.port.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    #[cfg(unix)]
    use std::{
        sync::{Mutex, MutexGuard},
        time::Instant,
    };

    /// `TTYPort::pair` is not safe to call from several threads at once, so the pseudo-terminal tests hold this lock.
    #[cfg(unix)]
    static PTY_LOCK: Mutex<()> = Mutex::new(());

    /// Returns a pseudo-terminal pair (master, slave) while holding [`PTY_LOCK`].
    #[cfg(unix)]
    fn a_pty_pair() -> (MutexGuard<'static, ()>, serialport::TTYPort, serialport::TTYPort) {
        let guard = PTY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (master, slave) = serialport::TTYPort::pair().unwrap();
        (guard, master, slave)
    }

    #[test]
    fn given_serial_port_path_when_port_does_not_exist_then_error() {
        let result = SerialPortChannel::new("/some/non/existent/port");
        assert!(matches!(result, Err(Error::Serial(_))));
    }

    #[test]
    fn given_serial_port_path_when_with_settle_and_port_does_not_exist_then_error() {
        let result = SerialPortChannel::with_settle("/some/non/existent/port", Duration::ZERO);
        assert!(matches!(result, Err(Error::Serial(_))));
    }

    #[test]
    fn given_default_settle_delay_when_compared_then_200_ms() {
        assert_eq!(DEFAULT_SETTLE_DELAY, Duration::from_millis(200));
    }

    #[cfg(unix)]
    #[test]
    fn given_a_settle_delay_when_channel_settled_then_wait_at_least_that_long() {
        let (_lock, _master, slave) = a_pty_pair();
        let start = Instant::now();
        SerialPortChannel::settled(Box::new(slave), Duration::from_millis(50)).unwrap();
        assert!(start.elapsed() >= Duration::from_millis(50));
    }

    #[cfg(unix)]
    #[test]
    fn given_an_open_channel_when_set_timeout_then_timeout_reports_new_value() {
        let (_lock, _master, slave) = a_pty_pair();
        let mut channel = SerialPortChannel::settled(Box::new(slave), Duration::ZERO).unwrap();

        channel.set_timeout(Duration::from_millis(250)).unwrap();

        assert_eq!(channel.timeout(), Duration::from_millis(250));
    }

    #[cfg(unix)]
    #[test]
    fn given_a_short_timeout_when_read_without_data_then_time_out_after_it() {
        let (_lock, _master, slave) = a_pty_pair();
        let mut channel = SerialPortChannel::settled(Box::new(slave), Duration::ZERO).unwrap();
        channel.set_timeout(Duration::from_millis(100)).unwrap();

        let start = Instant::now();
        let err = io::Read::read(&mut channel, &mut [0; 1]).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[cfg(unix)]
    #[test]
    fn given_stale_input_when_settled_then_input_is_discarded() {
        use std::io::{Read, Write};

        let (_lock, mut master, slave) = a_pty_pair();
        master.write_all(b"stale\n").unwrap();
        master.flush().unwrap();
        thread::sleep(Duration::from_millis(50));

        let mut channel = SerialPortChannel::settled(Box::new(slave), Duration::ZERO).unwrap();
        channel.set_timeout(Duration::from_millis(100)).unwrap();

        let err = channel.read(&mut [0; 1]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
    }
}
