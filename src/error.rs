//! Error type shared by the whole crate.
//!
//! Every fallible public function returns [`Result<T>`](Result), so callers can match on the [`Error`] variant to tell
//! apart I/O failures, corrupted frames, errors reported by the device and malformed data.
use std::{fmt, io};

use crate::frame::{Command, ErrorCode, FrameError};

/// Result alias using the crate [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// I/O error on the underlying channel (includes read timeouts).
    Io(io::Error),

    /// Failed to open or configure the serial port.
    Serial(SerialError),

    /// A received frame is malformed.
    Frame(FrameError),

    /// The device answered a command with an error frame.
    Device { command: Command, code: ErrorCode },

    /// The device answered with a frame for a different command.
    UnexpectedResponse { expected: Command, received: Command },

    /// The response payload could not be decoded.
    InvalidData(String),

    /// A parameter provided by the caller is not valid.
    InvalidParameter(String),

    /// The operation is not available on the current platform.
    Unsupported(&'static str),
}

/// Broad category of a [`SerialError`], so callers can react without depending on the serial port backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SerialErrorKind {
    /// The port does not exist or the device was disconnected.
    NoDevice,

    /// The OS denied access to the port.
    PermissionDenied,

    /// The port settings or the port name are not valid.
    InvalidInput,

    /// Any other failure.
    Other,
}

/// Failure to open or configure the serial port.
#[derive(Debug)]
pub struct SerialError {
    kind: SerialErrorKind,
    source: serialport::Error,
}

impl SerialError {
    /// Returns the category of this failure.
    pub fn kind(&self) -> SerialErrorKind {
        self.kind
    }

    /// Builds a serial error of `kind` with `message`, so code that handles serial failures can be tested without a
    /// port. `kind()` of the result returns `kind`.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn new(kind: SerialErrorKind, message: impl Into<String>) -> Self {
        let backend_kind = match kind {
            SerialErrorKind::NoDevice => serialport::ErrorKind::NoDevice,
            SerialErrorKind::PermissionDenied => serialport::ErrorKind::Io(io::ErrorKind::PermissionDenied),
            SerialErrorKind::InvalidInput => serialport::ErrorKind::InvalidInput,
            SerialErrorKind::Other => serialport::ErrorKind::Io(io::ErrorKind::Other),
        };
        SerialError::from(serialport::Error::new(backend_kind, message))
    }
}

impl fmt::Display for SerialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.source.fmt(f)
    }
}

impl std::error::Error for SerialError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<serialport::Error> for SerialError {
    fn from(source: serialport::Error) -> Self {
        let kind = match source.kind() {
            serialport::ErrorKind::NoDevice => SerialErrorKind::NoDevice,
            serialport::ErrorKind::InvalidInput => SerialErrorKind::InvalidInput,
            serialport::ErrorKind::Io(io::ErrorKind::NotFound) => SerialErrorKind::NoDevice,
            serialport::ErrorKind::Io(io::ErrorKind::PermissionDenied) => SerialErrorKind::PermissionDenied,
            _ => SerialErrorKind::Other,
        };
        SerialError { kind, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Serial(e) => write!(f, "serial port error: {}", e),
            Error::Frame(e) => write!(f, "invalid frame: {}", e),
            Error::Device { command, code } => {
                write!(f, "device error executing '{}': {} (0x{:X})", command, code, code)
            }
            Error::UnexpectedResponse { expected, received } => {
                write!(
                    f,
                    "unexpected response, expected {:?}, received {:?}",
                    expected, received
                )
            }
            Error::InvalidData(msg) => write!(f, "invalid data: {}", msg),
            Error::InvalidParameter(msg) => write!(f, "{}", msg),
            Error::Unsupported(what) => write!(f, "{} is not supported on this platform", what),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Serial(e) => Some(e),
            Error::Frame(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serialport::Error> for Error {
    fn from(e: serialport::Error) -> Self {
        Error::Serial(e.into())
    }
}

impl From<FrameError> for Error {
    fn from(e: FrameError) -> Self {
        Error::Frame(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_device_error_when_display_then_include_command_and_code() {
        let error = Error::Device {
            command: Command::GetIdn,
            code: ErrorCode::UnknownCmd,
        };
        assert_eq!(
            error.to_string(),
            "device error executing 'Get IDN': Unknown command (0x0324)"
        );
    }

    #[test]
    fn given_serial_errors_when_converted_then_map_to_kind() {
        let kind = |e: serialport::Error| SerialError::from(e).kind();
        let io = |k| serialport::Error::new(serialport::ErrorKind::Io(k), "");
        assert_eq!(
            kind(serialport::Error::new(serialport::ErrorKind::NoDevice, "")),
            SerialErrorKind::NoDevice
        );
        assert_eq!(kind(io(io::ErrorKind::NotFound)), SerialErrorKind::NoDevice);
        assert_eq!(
            kind(io(io::ErrorKind::PermissionDenied)),
            SerialErrorKind::PermissionDenied
        );
        assert_eq!(
            kind(serialport::Error::new(serialport::ErrorKind::InvalidInput, "")),
            SerialErrorKind::InvalidInput
        );
        assert_eq!(kind(io(io::ErrorKind::TimedOut)), SerialErrorKind::Other);
    }

    #[test]
    fn given_a_kind_when_build_serial_error_then_kind_round_trips() {
        for kind in [
            SerialErrorKind::NoDevice,
            SerialErrorKind::PermissionDenied,
            SerialErrorKind::InvalidInput,
            SerialErrorKind::Other,
        ] {
            let error = SerialError::new(kind, "port is busy");
            assert_eq!(error.kind(), kind);
            assert_eq!(error.to_string(), "port is busy");
        }
    }

    #[test]
    fn given_a_serial_error_when_wrapped_then_display_names_serial_port() {
        let error = Error::Serial(SerialError::new(SerialErrorKind::PermissionDenied, "access denied"));
        assert_eq!(error.to_string(), "serial port error: access denied");
    }

    #[test]
    fn given_an_io_error_when_source_then_return_inner_error() {
        let error = Error::from(io::Error::new(io::ErrorKind::TimedOut, "timeout"));
        assert!(std::error::Error::source(&error).is_some());
    }
}
