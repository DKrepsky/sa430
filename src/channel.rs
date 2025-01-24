//! # Channel Module
//!
//! This module provides an abstraction for communication channels, specifically focusing on serial port communication.
//! It defines a `Channel` trait that requires implementations for reading and writing, and a `SerialPortChannel` struct
//! that implements this trait using the `serialport` crate.
//!
//! ## Serial port settings
//!
//! - `SERIAL_PORT_BAUD_RATE`: The baud rate for the serial port communication, 926100 [bps].
//! - `SERIAL_PORT_STOP_BITS`: The number of stop bits used in the serial port communication, 1 [bit].
//! - `SERIAL_PORT_DATA_BITS`: The number of data bits used in the serial port communication, 8 [bits].
//! - `SERIAL_PORT_PARITY`: The parity setting for the serial port communication, none.
//! - `SERIAL_PORT_FLOW_CONTROL`: The flow control setting for the serial port communication, none.
//! - `SERIAL_PORT_TIMEOUT`: The timeout duration for the serial port communication, 5 [seconds].
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::channel::{Channel, SerialPortChannel};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut channel = SerialPortChannel::new("/dev/ttyUSB0")?;
//!     let reader = channel.reader();
//!     let writer = channel.writer();
//!     // Use reader and writer for communication
//!     Ok(())
//! }
//! ```
use std::{error, io, time::Duration};

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

const SERIAL_PORT_BAUD_RATE: u32 = 926100;
const SERIAL_PORT_STOP_BITS: StopBits = StopBits::One;
const SERIAL_PORT_DATA_BITS: DataBits = DataBits::Eight;
const SERIAL_PORT_PARITY: Parity = Parity::None;
const SERIAL_PORT_FLOW_CONTROL: FlowControl = FlowControl::None;
const SERIAL_PORT_TIMEOUT: Duration = Duration::from_secs(5);

/// ### `Channel`
///
/// A trait that represents a communication channel. It requires implementations for reading and writing.
pub trait Channel: io::Read + io::Write {
    /// Returns a mutable reference to the reader part of the channel.
    fn reader(&mut self) -> &mut dyn io::Read;

    /// Returns a mutable reference to the writer part of the channel.
    fn writer(&mut self) -> &mut dyn io::Write;
}

/// ### `SerialPortChannel`
///
/// A struct that implements the `Channel` trait using a serial port.
/// It encapsulates a serial port and provides methods to open and manage the port.
pub struct SerialPortChannel {
    port: Box<dyn SerialPort>,
}

impl SerialPortChannel {
    /// Creates a new `SerialPortChannel` by opening the specified serial port with default settings used by the SA430 hardware.
    pub fn new(port_name: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(SerialPortChannel {
            port: SerialPortChannel::open(port_name)?,
        })
    }
    /// Opens the specified serial port with the predefined settings.
    fn open(port_name: &str) -> Result<Box<dyn SerialPort>, serialport::Error> {
        let builder = serialport::new(port_name, SERIAL_PORT_BAUD_RATE)
            .stop_bits(SERIAL_PORT_STOP_BITS)
            .data_bits(SERIAL_PORT_DATA_BITS)
            .parity(SERIAL_PORT_PARITY)
            .flow_control(SERIAL_PORT_FLOW_CONTROL)
            .timeout(SERIAL_PORT_TIMEOUT);

        builder.open()
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

impl Channel for SerialPortChannel {
    fn reader(&mut self) -> &mut dyn io::Read {
        self
    }

    fn writer(&mut self) -> &mut dyn io::Write {
        self
    }
}

pub mod fixtures {
    //! # Fixtures Module
    //!
    //! This module provides mock implementations of the `Channel` trait for testing purposes.
    //! It includes the `MockChannel` struct which can be used to simulate reading from and writing to a channel.
    //!
    //! ## Usage Example
    //!
    //! ```rust
    //! use crate::channel::fixtures::MockChannel;
    //!
    //! fn main() {
    //!     let mut mock_channel = MockChannel::new();
    //!     mock_channel.add_response(b"Hello, world!");
    //!     // Use mock_channel for testing
    //! }
    //! ```
    use super::*;

    /// ### `MockChannel`
    ///
    /// A struct that simulates a communication channel by using in-memory buffers for reading and writing.
    pub struct MockChannel {
        /// A `Vec<u8>` that acts as the buffer for incoming data.
        pub read_buffer: Vec<u8>,
        /// A `Vec<u8>` that acts as the buffer for outgoing data.
        pub write_buffer: Vec<u8>,
    }

    impl MockChannel {
        /// Creates a new `MockChannel` with empty read and write buffers.
        pub fn new() -> Self {
            MockChannel {
                read_buffer: Vec::new(),
                write_buffer: Vec::new(),
            }
        }

        /// Adds a response to the read buffer, simulating incoming data.
        pub fn add_response(&mut self, response: &[u8]) {
            self.read_buffer.extend(response);
        }
    }

    impl io::Write for MockChannel {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.write_buffer.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.write_buffer.flush()
        }
    }

    impl io::Read for MockChannel {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            // Copy buf.len() bytes from the read buffer and remove the copied data to simulate reading from the channel
            for i in 0..buf.len() {
                buf[i] = self.read_buffer.remove(0);
            }
            Ok(buf.len())
        }
    }

    impl Channel for MockChannel {
        fn reader(&mut self) -> &mut dyn std::io::Read {
            self
        }

        fn writer(&mut self) -> &mut dyn std::io::Write {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_serial_port_path_when_port_does_not_exist_then_error() {
        let port_name = "/some/non/existent/port";
        let result = SerialPortChannel::new(port_name);
        assert!(result.is_err());
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "No such file or directory");
        } else {
            panic!("Expected an error");
        }
    }
}
