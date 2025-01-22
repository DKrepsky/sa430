use std::{
    error::Error,
    fmt::{self, Display},
};

use super::crc::crc16;

/// Magic value for the SA430 protocol.
pub const FRAME_MAGIC_VALUE: u8 = 0x2A;

/// Frame magic value index.
pub const FRAME_MAGIC_INDEX: usize = 0;

/// Frame data length index.
pub const FRAME_DATA_LENGTH_INDEX: usize = 1;

/// Frame command index.
pub const FRAME_COMMAND_INDEX: usize = 2;

/// Frame data index.
pub const FRAME_DATA_INDEX: usize = 3;

/// Frame header size (magic, length, command).
pub const FRAME_HEADER_SIZE: usize = 3;

/// Frame CRC size.
pub const FRAME_CRC_SIZE: usize = 2;

/// SA430 command codes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    /// Unknown
    #[default]
    Unknown = 0x00,

    /// Get IDN
    GetIdn = 0x01,

    /// Get Hardware Serial Number
    GetHwSerNr = 0x02,

    /// Hardware Reset (PUC)
    HwReset = 0x03,

    /// Identify hardware by blinking LED
    BlinkLed = 0x04,

    /// Get core version
    GetCoreVer = 0x05,

    /// Error
    GetLastError = 0x06,

    /// Unknown
    Sync = 0x07,

    /// Get Spec version
    GetSpecVer = 0x14,

    /// Set Start Frequency `fstart`
    SetFStart = 0x15,

    /// Set Stop Frequency `fstop`
    SetFStop = 0x16,

    /// Set Step Frequency `fstep`
    SetFStep = 0x17,

    /// Unknown
    SetFrq = 0x18,

    /// Set Rx Filter Bandwidth
    SetRbw = 0x19,

    /// Set DC value for the balun (unknown)
    SetDac = 0x1A,

    /// Set gain of the Rx path
    SetGain = 0x1B,

    /// Set Intermediate Frequency
    SetIf = 0x1C,

    /// Setup system for spectrum measurement
    InitParameter = 0x1E,

    /// Measure spectrum with defined parameters
    GetSpecNoInit = 0x1F,

    /// Get prod version
    GetProdVer = 0x3C,

    /// Unknown
    SetProdFwInit = 0x3D,

    /// Unknown
    GetTemp = 0x3E,

    /// Set hardware id
    SetHwId = 0x3F,

    /// Get Hardware id
    GetHwId = 0x40,

    /// Boot count
    GetBootCnt = 0x41,

    /// 0=Off, 1=26MHz, 2=RF Freq.  (next bytes)
    SetFout = 0x42,

    /// Set frequency, incl. temp/cal versions
    SetFxtal = 0x43,

    /// Get frequency, incl. temp/cal versions
    GetFxtal = 0x44,

    /// f, gain, repetition count
    SweepEdc = 0x45,

    /// Unknown
    GetChipTlv = 0x49,

    /// Send address and size, get flash content
    FlashRead = 0x0A,

    /// Unknown
    FlashWrite = 0x0B,

    /// Unknown
    FlashErase = 0x0C,

    /// Unknown
    FlashGetCrc = 0x0D,

    /// Frame Error
    FrameError = 0xFF,
}

impl From<u8> for Command {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Command::Unknown,
            0x01 => Command::GetIdn,
            0x02 => Command::GetHwSerNr,
            0x03 => Command::HwReset,
            0x04 => Command::BlinkLed,
            0x05 => Command::GetCoreVer,
            0x06 => Command::GetLastError,
            0x07 => Command::Sync,
            0x14 => Command::GetSpecVer,
            0x15 => Command::SetFStart,
            0x16 => Command::SetFStop,
            0x17 => Command::SetFStep,
            0x18 => Command::SetFrq,
            0x19 => Command::SetRbw,
            0x1A => Command::SetDac,
            0x1B => Command::SetGain,
            0x1C => Command::SetIf,
            0x1E => Command::InitParameter,
            0x1F => Command::GetSpecNoInit,
            0x3C => Command::GetProdVer,
            0x3D => Command::SetProdFwInit,
            0x3E => Command::GetTemp,
            0x3F => Command::SetHwId,
            0x40 => Command::GetHwId,
            0x41 => Command::GetBootCnt,
            0x42 => Command::SetFout,
            0x43 => Command::SetFxtal,
            0x44 => Command::GetFxtal,
            0x45 => Command::SweepEdc,
            0x49 => Command::GetChipTlv,
            0x0A => Command::FlashRead,
            0x0B => Command::FlashWrite,
            0x0C => Command::FlashErase,
            0x0D => Command::FlashGetCrc,
            0xFF => Command::FrameError,
            _ => Command::Unknown,
        }
    }
}

/// SA430 Error codes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorCode {
    NoError = 0x0000,
    CmdBufferOverflow = 0x0320,
    WrongCmdLength = 0x0321,
    CmdAborted = 0x0322,
    LostCmd = 0x0323,
    CmdUnknown = 0x0324,
    TooMuchDataRequestedByUserFunction = 0x0325,
    RestoreProgramCounter = 0x0326,
    BufferPosOutOfRange = 0x0327,
    EeqBufferOverflow = 0x0328,
    WrongCrcLowByte = 0x0329,
    WrongCrcHighByte = 0x032A,
    RestoreFromPacketError = 0x032C,
    NoFrameStart = 0x032D,
    WrongPktLength = 0x032E,
    PacketIncomplete = 0x032F,
    PacketError = 0x0330,
    StupidPacketHandler = 0x0331,
    BufferOverflow = 0x0352,
    BufferUnderrun = 0x0353,
    FlashNotErased = 0x044C,
    FlashMismatch = 0x044D,
    RssiValidFlagNotSet = 0x04B0,
    PllNotSettled = 0x04B1,
    #[default]
    Unknown = 0xFFFF,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            ErrorCode::NoError => "OK",
            ErrorCode::CmdBufferOverflow => "Command buffer overflow",
            ErrorCode::WrongCmdLength => "Wrong command length",
            ErrorCode::CmdAborted => "Command aborted",
            ErrorCode::LostCmd => "Lost command",
            ErrorCode::CmdUnknown => "Unknown command",
            ErrorCode::TooMuchDataRequestedByUserFunction => {
                "Too much data requested by user function"
            }
            ErrorCode::RestoreProgramCounter => "Restore program counter",
            ErrorCode::BufferPosOutOfRange => "Buffer position out of range",
            ErrorCode::EeqBufferOverflow => "EEQ buffer overflow",
            ErrorCode::WrongCrcLowByte => "Wrong CRC low byte",
            ErrorCode::WrongCrcHighByte => "Wrong CRC high byte",
            ErrorCode::RestoreFromPacketError => "Restore from packet error",
            ErrorCode::NoFrameStart => "No frame start",
            ErrorCode::WrongPktLength => "Wrong packet length",
            ErrorCode::PacketIncomplete => "Packet incomplete",
            ErrorCode::PacketError => "Packet error",
            ErrorCode::StupidPacketHandler => "Stupid packet handler",
            ErrorCode::BufferOverflow => "Buffer overflow",
            ErrorCode::BufferUnderrun => "Buffer underrun",
            ErrorCode::FlashNotErased => "Flash not erased",
            ErrorCode::FlashMismatch => "Flash mismatch",
            ErrorCode::RssiValidFlagNotSet => "RSSI valid flag not set",
            ErrorCode::PllNotSettled => "PLL not settled",
            ErrorCode::Unknown => "Unknown error",
        };
        write!(f, "{}", description)
    }
}

impl From<Vec<u8>> for ErrorCode {
    fn from(value: Vec<u8>) -> Self {
        let code = u16::from_be_bytes([value[0], value[1]]);
        match code {
            0x0000 => ErrorCode::NoError,
            0x0320 => ErrorCode::CmdBufferOverflow,
            0x0321 => ErrorCode::WrongCmdLength,
            0x0322 => ErrorCode::CmdAborted,
            0x0323 => ErrorCode::LostCmd,
            0x0324 => ErrorCode::CmdUnknown,
            0x0325 => ErrorCode::TooMuchDataRequestedByUserFunction,
            0x0326 => ErrorCode::RestoreProgramCounter,
            0x0327 => ErrorCode::BufferPosOutOfRange,
            0x0328 => ErrorCode::EeqBufferOverflow,
            0x0329 => ErrorCode::WrongCrcLowByte,
            0x032A => ErrorCode::WrongCrcHighByte,
            0x032C => ErrorCode::RestoreFromPacketError,
            0x032D => ErrorCode::NoFrameStart,
            0x032E => ErrorCode::WrongPktLength,
            0x032F => ErrorCode::PacketIncomplete,
            0x0330 => ErrorCode::PacketError,
            0x0331 => ErrorCode::StupidPacketHandler,
            0x0352 => ErrorCode::BufferOverflow,
            0x0353 => ErrorCode::BufferUnderrun,
            0x044C => ErrorCode::FlashNotErased,
            0x044D => ErrorCode::FlashMismatch,
            0x04B0 => ErrorCode::RssiValidFlagNotSet,
            0x04B1 => ErrorCode::PllNotSettled,
            _ => ErrorCode::Unknown,
        }
    }
}

/// Error types for the SA430 protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrameError {
    /// Invalid magic value (current value).
    InvalidMagicValue(u8),

    /// Invalid frame length (expected, current).
    InvalidFrameLength(u8, u8),

    /// Invalid CRC (expected, current).
    InvalidCrc(u16, u16),
}

impl Error for FrameError {}

impl Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::InvalidMagicValue(current) => {
                write!(
                    f,
                    "Invalid magic value, expected 0x{:02X}, current: 0x{:02X}",
                    FRAME_MAGIC_VALUE, current
                )
            }
            FrameError::InvalidFrameLength(expected, current) => {
                write!(
                    f,
                    "Invalid frame length, expected 0x{:02X}, current: 0x{:02X}",
                    expected, current
                )
            }
            FrameError::InvalidCrc(expected, current) => {
                write!(
                    f,
                    "Invalid CRC, expected: 0x{:04X}, current: 0x{:04X}",
                    expected, current
                )
            }
        }
    }
}

/// SA430 Frame structure.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    cmd: Command,
    data: Vec<u8>,
}

impl Frame {
    /// Creates a new frame with default values.
    pub fn new(cmd: Command, data: Vec<u8>) -> Self {
        Frame { cmd, data }
    }

    pub fn cmd(&self) -> Command {
        self.cmd
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FrameError> {
        Frame::validate(bytes)?;
        Frame::parse(bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![FRAME_MAGIC_VALUE, self.data.len() as u8, self.cmd as u8];
        bytes.append(&mut self.data.clone());
        let crc = crc16(&bytes);
        bytes.append(&mut crc.to_be_bytes().to_vec());
        bytes
    }

    pub fn to_error_code(&self) -> Option<ErrorCode> {
        match self.cmd() {
            Command::GetLastError => Some(self.data.clone().into()),
            _ => None,
        }
    }

    fn validate(bytes: &[u8]) -> Result<(), FrameError> {
        if bytes[FRAME_MAGIC_INDEX] != FRAME_MAGIC_VALUE {
            return Err(FrameError::InvalidMagicValue(bytes[FRAME_MAGIC_INDEX]));
        }

        if bytes.len() - 5 != bytes[FRAME_DATA_LENGTH_INDEX] as usize {
            return Err(FrameError::InvalidFrameLength(
                bytes[FRAME_DATA_LENGTH_INDEX],
                (bytes.len() - 5) as u8,
            ));
        }

        let frame_crc = u16::from_be_bytes([bytes[bytes.len() - 2], bytes[bytes.len() - 1]]);
        let computed_crc = crc16(&bytes[..bytes.len() - 2]);
        if frame_crc != computed_crc {
            return Err(FrameError::InvalidCrc(frame_crc, computed_crc));
        }

        Ok(())
    }

    fn parse(bytes: &[u8]) -> Result<Self, FrameError> {
        let cmd = Command::from(bytes[FRAME_COMMAND_INDEX]);
        let data = bytes[FRAME_DATA_INDEX..bytes.len() - FRAME_CRC_SIZE].to_vec();

        Ok(Frame { cmd, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_bytes_when_from_bytes_then_return_frame() {
        let bytes = vec![0x2A, 0x04, 0x0A, 0xD4, 0x00, 0x00, 0x0A, 0xCD, 0xAD];
        let frame = Frame::from_bytes(&bytes).unwrap();
        assert_eq!(frame.cmd(), Command::FlashRead);
        assert_eq!(frame.data(), vec![0xD4, 0x00, 0x00, 0x0A].as_slice());
    }

    #[test]
    fn given_a_wrong_magic_number_when_from_bytes_then_return_error() {
        let bytes = vec![0x2B, 0x04, 0x0A, 0xD4, 0x00, 0x00, 0x0A, 0xCD, 0xAD];
        let result = Frame::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FrameError::InvalidMagicValue(0x2B));
    }

    #[test]
    fn given_a_wrong_frame_length_when_from_bytes_then_return_error() {
        let bytes = vec![0x2A, 0x03, 0x0A, 0xD4, 0x00, 0x00, 0x0A, 0xCD, 0xAD];
        let result = Frame::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            FrameError::InvalidFrameLength(0x03, 0x04)
        );
    }

    #[test]
    fn given_a_wrong_crc_when_from_bytes_then_return_error() {
        let bytes = vec![0x2A, 0x04, 0x0A, 0xD4, 0x00, 0x00, 0x0A, 0xCD, 0xAE];
        let result = Frame::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FrameError::InvalidCrc(0xCDAE, 0xCDAD));
    }

    #[test]
    fn given_a_frame_when_to_bytes_then_return_bytes() {
        let frame = Frame::new(Command::FlashRead, vec![0xD4, 0x00, 0x00, 0x0A]);
        let bytes = frame.to_bytes();
        assert_eq!(
            bytes,
            vec![0x2A, 0x04, 0x0A, 0xD4, 0x00, 0x00, 0x0A, 0xCD, 0xAD]
        );
    }

    #[test]
    fn given_an_error_when_to_error_code_then_return_error_code() {
        let frame = Frame::new(Command::GetLastError, vec![0x03, 0x20]);
        let error_code = frame.to_error_code().unwrap();
        assert_eq!(error_code, ErrorCode::CmdBufferOverflow);
    }
}
