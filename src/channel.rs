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
//! - Read timeout: 5 seconds.
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
use std::{io, time::Duration};

use serialport::{ClearBuffer, DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::error::Result;

const SERIAL_PORT_BAUD_RATE: u32 = 926100;
const SERIAL_PORT_STOP_BITS: StopBits = StopBits::One;
const SERIAL_PORT_DATA_BITS: DataBits = DataBits::Eight;
const SERIAL_PORT_PARITY: Parity = Parity::None;
const SERIAL_PORT_FLOW_CONTROL: FlowControl = FlowControl::None;
const SERIAL_PORT_TIMEOUT: Duration = Duration::from_secs(5);

/// A bidirectional, blocking byte stream connected to a device.
pub trait Channel: io::Read + io::Write {}

impl<T: io::Read + io::Write + ?Sized> Channel for T {}

/// A [`Channel`] backed by a serial port configured for the SA430.
pub struct SerialPortChannel {
    port: Box<dyn SerialPort>,
}

impl SerialPortChannel {
    /// Opens `port_name` with the settings used by the SA430 hardware.
    ///
    /// Any bytes already waiting in the input buffer (e.g. left over from an interrupted session) are discarded, so
    /// they are not mistaken for a response to the next command.
    pub fn new(port_name: &str) -> Result<Self> {
        let port = serialport::new(port_name, SERIAL_PORT_BAUD_RATE)
            .stop_bits(SERIAL_PORT_STOP_BITS)
            .data_bits(SERIAL_PORT_DATA_BITS)
            .parity(SERIAL_PORT_PARITY)
            .flow_control(SERIAL_PORT_FLOW_CONTROL)
            .timeout(SERIAL_PORT_TIMEOUT)
            .open()?;

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

    #[test]
    fn given_serial_port_path_when_port_does_not_exist_then_error() {
        let result = SerialPortChannel::new("/some/non/existent/port");
        assert!(matches!(result, Err(Error::Serial(_))));
    }
}
