use std::error::Error;

use super::channel::*;
use super::frame::*;
use super::protocol::*;
use crate::parser::ByteArrayParser;

/// Start address of the calibration data in the flash memory.
const FLASH_PROG_HEADER_ADDR: u16 = 0xD400;

/// Size of the calibration data in the flash memory.
const FLASH_PROG_HEADER_SIZE: u16 = 0x000A;

/// Expected type of the calibration data in the flash memory.
const FLASH_PROG_HEADER_TYPE: u16 = 0x003E;

/// Start address of the calibration data in the flash memory.
const FLASH_CALIBRATION_ADDR: u16 = 0xD40A;

/// Size of the calibration data in the flash memory.
const FLASH_CALIBRATION_SIZE: u16 = 0x0687;

/// Represents a frequency range with start and stop frequencies and number of samples.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FrequencyRange {
    /// Start frequency in Hz.
    f_start: u32,
    /// Stop frequency in Hz.
    f_stop: u32,
    /// Number of samples.
    samples: u32,
}

impl From<&[u8; 12]> for FrequencyRange {
    fn from(value: &[u8; 12]) -> Self {
        Self {
            f_start: u32::from_be_bytes(value[0..4].try_into().unwrap()),
            f_stop: u32::from_be_bytes(value[4..8].try_into().unwrap()),
            samples: u32::from_be_bytes(value[8..12].try_into().unwrap()),
        }
    }
}

/// Represents a reference level with value and gain.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RefLevel {
    /// Reference level value.
    value: u8,
    /// Gain value.
    gain: u8,
}

impl From<&[u8; 2]> for RefLevel {
    fn from(value: &[u8; 2]) -> Self {
        Self {
            value: value[0],
            gain: value[1],
        }
    }
}

/// Represents a frequency gain with reference level index and array of gain values.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FrequencyGain {
    /// Reference level index.
    ref_level_index: u8,
    /// Array of gain values.
    gains: [f64; 8],
}

impl From<&[u8; 65]> for FrequencyGain {
    fn from(value: &[u8; 65]) -> Self {
        let ref_level_index = value[0];
        let mut gains = [0.0; 8];

        for (i, gain) in gains.iter_mut().enumerate() {
            let offset = 1 + i * 8;
            let bytes: [u8; 8] = value[offset..offset + 8].try_into().unwrap();
            *gain = f64::from_be_bytes(bytes);
        }

        Self { ref_level_index, gains }
    }
}

/// Represents the calibration data of the device.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Calibration {
    /// Hardware ID.
    pub hardware_id: u32,

    /// Serial number.
    pub serial_number: [u8; 16],

    /// Format version.
    pub format_version: u16,

    /// Software version.
    pub software_version: u16,

    /// Production side.
    pub production_side: u8,

    /// Calibration date.
    pub calibration_date: [u8; 16],

    /// Crystal frequency in Hz.
    pub xtal_freq_hz: u32,

    /// Crystal frequency error in ppm.
    pub xtal_freq_error_ppm: u16,

    /// Calibration temperature start.
    pub calibration_temperature_start: [u8; 6],

    /// Calibration temperature stop.
    pub calibration_temperature_stop: [u8; 6],

    /// Reference levels.
    pub ref_levels: [RefLevel; 8],

    /// Frequency ranges.
    pub frq_ranges: [FrequencyRange; 3],

    /// Frequency gains tables.
    pub frq_gains_tables: [[FrequencyGain; 8]; 3],
}

impl TryFrom<&[u8]> for Calibration {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut cal = Calibration::default();
        let mut parser = ByteArrayParser::new(value);

        cal.format_version = parser.take_u16()?;
        cal.calibration_date = parser.take_bytes(16)?.try_into()?;
        cal.software_version = parser.take_u16()?;
        cal.production_side = parser.take_u8()?;

        for i in 0..3 {
            cal.frq_ranges[i] = FrequencyRange::from(&parser.take_bytes(12)?.try_into()?);
        }

        for i in 0..8 {
            cal.ref_levels[i] = RefLevel::from(&parser.take_bytes(2)?.try_into()?);
        }

        cal.hardware_id = parser.take_u32()?;
        cal.serial_number = parser.take_bytes(16)?.try_into()?;
        cal.xtal_freq_hz = parser.take_u32()?;
        cal.xtal_freq_error_ppm = parser.take_u16()?;
        cal.calibration_temperature_start = parser.take_bytes(6)?.try_into()?;
        cal.calibration_temperature_stop = parser.take_bytes(6)?.try_into()?;

        for i in 0..3 {
            for j in 0..8 {
                cal.frq_gains_tables[i][j] = FrequencyGain::from(&parser.take_bytes(65)?.try_into()?);
            }
        }

        Ok(cal)
    }
}

/// Represents a program header in the flash memory.
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ProgHeader {
    pub mem_start_address: u16,
    pub mem_length: u16,
    pub mem_type: u16,
    pub type_version: u16,
    pub crc: u16,
}

impl From<&[u8]> for ProgHeader {
    fn from(bytes: &[u8]) -> Self {
        ProgHeader {
            mem_start_address: u16::from_le_bytes([bytes[0], bytes[1]]),
            mem_length: u16::from_le_bytes([bytes[2], bytes[3]]),
            mem_type: u16::from_le_bytes([bytes[4], bytes[5]]),
            type_version: u16::from_le_bytes([bytes[6], bytes[7]]),
            crc: u16::from_le_bytes([bytes[8], bytes[9]]),
        }
    }
}

/// SA430 device proxy.
///
/// This class provides a high-level API to access the device functionality, such as reading the device information,
/// calibration data, and spectrum data.
/// The device is initialized with a channel that provides the communication link with the device, usually a serial port,
/// but it can be any other type of channel that implements the `Channel` trait.
///
/// # Note
///  - All methods are blocking.
pub struct Sa430 {
    channel: Box<dyn Channel>,
    calibration: Option<Calibration>,
}

impl Sa430 {
    /// Creates a new SA430 device with the specified channel.
    pub fn new(channel: Box<dyn Channel>) -> Self {
        Sa430 {
            channel,
            calibration: None,
        }
    }

    /// Gets the device identification string.
    pub fn idn(&mut self) -> Result<String, Box<dyn Error>> {
        get_string(self.channel.as_mut(), Command::GetIdn)
    }

    /// Gets the device model.
    pub fn serial_number(&mut self) -> Result<u32, Box<dyn Error>> {
        get_u32(self.channel.as_mut(), Command::GetSerialNumber)
    }

    /// Gets the device model.
    pub fn core_version(&mut self) -> Result<String, Box<dyn Error>> {
        get_u16(self.channel.as_mut(), Command::GetCoreVersion).map(|v| (format!("{}.{}", v >> 8, v & 0xFF)))
    }

    /// Gets the device model.
    pub fn spectrum_version(&mut self) -> Result<String, Box<dyn Error>> {
        get_u16(self.channel.as_mut(), Command::GetSpectrumVersion).map(|v| (format!("{}.{}", v >> 8, v & 0xFF)))
    }

    /// Blink the device LED.
    pub fn blink(&mut self) -> Result<(), Box<dyn Error>> {
        exec(self.channel.as_mut(), Command::BlinkLed)
    }

    /// Reboot the device.
    pub fn reboot(&mut self) -> Result<(), Box<dyn Error>> {
        exec(self.channel.as_mut(), Command::HardwareReset)
    }

    /// Gets the device calibration data.
    ///
    /// Result is cached for subsequent calls.
    pub fn calibration(&mut self) -> Result<&Calibration, Box<dyn Error>> {
        if self.calibration.is_none() {
            self.calibration = Some(self.fetch_calibration()?);
        }

        return Ok(self.calibration.as_ref().unwrap());
    }

    /// Prettifies the calibration data version.
    pub fn calibration_version(&mut self) -> Result<String, Box<dyn Error>> {
        self.calibration()
            .map(|c| format!("{}.{}", c.format_version >> 8, c.format_version & 0xFF))
    }

    /// Prettifies the calibration data date.
    pub fn calibration_date(&mut self) -> Result<String, Box<dyn Error>> {
        self.calibration()
            .map(|c| String::from_utf8_lossy(&c.calibration_date).to_string())
    }

    fn fetch_calibration(&mut self) -> Result<Calibration, Box<dyn Error>> {
        self.check_prog_header()?;
        self.read_calibration()
    }

    fn check_prog_header(&mut self) -> Result<(), Box<dyn Error>> {
        let prog_header_vec = read_flash(self.channel.as_mut(), FLASH_PROG_HEADER_ADDR, FLASH_PROG_HEADER_SIZE)?;
        let prog_header: ProgHeader = prog_header_vec.as_slice().into();
        if prog_header.mem_type != FLASH_PROG_HEADER_TYPE {
            let message = format!(
                "Invalid flash memory type, expected: {}, got: {}",
                FLASH_PROG_HEADER_TYPE, prog_header.mem_type
            );
            return Err(message.into());
        }
        Ok(())
    }

    fn read_calibration(&mut self) -> Result<Calibration, Box<dyn Error>> {
        let calibration_vec = read_flash(self.channel.as_mut(), FLASH_CALIBRATION_ADDR, FLASH_CALIBRATION_SIZE)?;
        calibration_vec.as_slice().try_into()
    }
}
