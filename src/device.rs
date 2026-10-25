//! High-level API for an SA430 device.
//!
//! [`Sa430`] wraps a [`Channel`] and exposes the device functionality (identification, LED, reset, calibration data)
//! as plain methods. Calibration data is decoded into [`Calibration`].
use crate::channel::{Channel, SerialPortChannel};
use crate::error::{Error, Result};
use crate::frame::Command;
use crate::parser::ByteArrayParser;
use crate::protocol::*;
use crate::spectrum::{Measurement, SweepRequest};

/// Start address of the program header (describes the calibration block) in flash memory.
const FLASH_PROG_HEADER_ADDR: u16 = 0xD400;

/// Size of the program header in flash memory.
const FLASH_PROG_HEADER_SIZE: u16 = 0x000A;

/// Memory type the program header must declare for the calibration block.
const FLASH_PROG_HEADER_TYPE: u16 = 0x003E;

/// Start address of the calibration data in flash memory.
const FLASH_CALIBRATION_ADDR: u16 = 0xD40A;

/// Size of the calibration data in flash memory.
const FLASH_CALIBRATION_SIZE: u16 = 0x0687;

/// Password the firmware requires as payload of [`Command::HardwareReset`].
const HARDWARE_RESET_PASSWORD: [u8; 2] = [0xDE, 0xAD];

/// A frequency range with start and stop frequencies and number of samples.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FrequencyRange {
    /// Start frequency in Hz.
    pub f_start: u32,
    /// Stop frequency in Hz.
    pub f_stop: u32,
    /// Number of samples.
    pub samples: u32,
}

impl FrequencyRange {
    fn parse(parser: &mut ByteArrayParser) -> Result<Self> {
        Ok(Self {
            f_start: parser.take_u32()?,
            f_stop: parser.take_u32()?,
            samples: parser.take_u32()?,
        })
    }
}

/// A reference level with its gain setting.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RefLevel {
    /// Reference level value.
    pub value: u8,
    /// Gain value.
    pub gain: u8,
}

impl RefLevel {
    fn parse(parser: &mut ByteArrayParser) -> Result<Self> {
        Ok(Self {
            value: parser.take_u8()?,
            gain: parser.take_u8()?,
        })
    }
}

/// Gain correction polynomial for one reference level.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FrequencyGain {
    /// Reference level index.
    pub ref_level_index: u8,
    /// Gain values.
    pub gains: [f64; 8],
}

impl FrequencyGain {
    fn parse(parser: &mut ByteArrayParser) -> Result<Self> {
        let ref_level_index = parser.take_u8()?;
        let mut gains = [0.0; 8];
        for gain in gains.iter_mut() {
            *gain = parser.take_f64()?;
        }
        Ok(Self { ref_level_index, gains })
    }
}

/// Calibration data stored in the device flash.
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

    /// Calibration date, as a NUL padded ASCII string.
    pub calibration_date: [u8; 16],

    /// Crystal frequency in Hz.
    pub xtal_freq_hz: u32,

    /// Crystal frequency error in ppm (signed).
    pub xtal_freq_error_ppm: i16,

    /// Calibration temperature start.
    pub calibration_temperature_start: [u8; 6],

    /// Calibration temperature stop.
    pub calibration_temperature_stop: [u8; 6],

    /// Reference levels.
    pub ref_levels: [RefLevel; 8],

    /// Frequency ranges.
    pub frq_ranges: [FrequencyRange; 3],

    /// Frequency gains tables, one per frequency range.
    pub frq_gains_tables: [[FrequencyGain; 8]; 3],
}

impl Calibration {
    /// Format version as `major.minor`.
    pub fn version(&self) -> String {
        format_version(self.format_version)
    }

    /// Calibration date as text, without the NUL padding.
    pub fn date(&self) -> String {
        String::from_utf8_lossy(&self.calibration_date)
            .trim_end_matches('\0')
            .to_string()
    }
}

impl TryFrom<&[u8]> for Calibration {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let mut cal = Calibration::default();
        let mut parser = ByteArrayParser::new(value);

        cal.format_version = parser.take_u16()?;
        cal.calibration_date = parser.take_array()?;
        cal.software_version = parser.take_u16()?;
        cal.production_side = parser.take_u8()?;

        for range in cal.frq_ranges.iter_mut() {
            *range = FrequencyRange::parse(&mut parser)?;
        }

        for level in cal.ref_levels.iter_mut() {
            *level = RefLevel::parse(&mut parser)?;
        }

        cal.hardware_id = parser.take_u32()?;
        cal.serial_number = parser.take_array()?;
        cal.xtal_freq_hz = parser.take_u32()?;
        cal.xtal_freq_error_ppm = parser.take_i16()?;
        cal.calibration_temperature_start = parser.take_array()?;
        cal.calibration_temperature_stop = parser.take_array()?;

        for table in cal.frq_gains_tables.iter_mut() {
            for gain in table.iter_mut() {
                *gain = FrequencyGain::parse(&mut parser)?;
            }
        }

        Ok(cal)
    }
}

/// Program header in flash memory. Unlike the rest of the protocol, it is stored little-endian (MSP430 native order).
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ProgHeader {
    pub mem_start_address: u16,
    pub mem_length: u16,
    pub mem_type: u16,
    pub type_version: u16,
    pub crc: u16,
}

impl TryFrom<&[u8]> for ProgHeader {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        let mut parser = ByteArrayParser::new(bytes);
        Ok(ProgHeader {
            mem_start_address: parser.take_u16_le()?,
            mem_length: parser.take_u16_le()?,
            mem_type: parser.take_u16_le()?,
            type_version: parser.take_u16_le()?,
            crc: parser.take_u16_le()?,
        })
    }
}

/// SA430 device proxy.
///
/// Provides a high-level API to the device functionality, such as reading the device information and calibration
/// data. The device talks over a [`Channel`], usually a serial port (see [`Sa430::open`]), but any `Read + Write`
/// type works, which makes it easy to test.
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

    /// Opens the device connected to serial port `port_name`.
    pub fn open(port_name: &str) -> Result<Self> {
        Ok(Self::new(Box::new(SerialPortChannel::new(port_name)?)))
    }

    /// Gets the device identification string.
    pub fn idn(&mut self) -> Result<String> {
        get_string(self.channel.as_mut(), Command::GetIdn)
    }

    /// Gets the hardware serial number.
    pub fn serial_number(&mut self) -> Result<u32> {
        get_u32(self.channel.as_mut(), Command::GetSerialNumber)
    }

    /// Gets the core firmware version as `major.minor`.
    pub fn core_version(&mut self) -> Result<String> {
        get_u16(self.channel.as_mut(), Command::GetCoreVersion).map(format_version)
    }

    /// Gets the spectrum firmware version as `major.minor`.
    pub fn spectrum_version(&mut self) -> Result<String> {
        get_u16(self.channel.as_mut(), Command::GetSpectrumVersion).map(format_version)
    }

    /// Blinks the device LED, to identify it.
    pub fn blink(&mut self) -> Result<()> {
        exec(self.channel.as_mut(), Command::BlinkLed)
    }

    /// Performs a hardware reset.
    pub fn reboot(&mut self) -> Result<()> {
        exec_with_data(self.channel.as_mut(), Command::HardwareReset, &HARDWARE_RESET_PASSWORD)
    }

    /// Measures the spectrum described by `request`.
    ///
    /// Configures the device, runs the sweep and applies the calibration offset to the samples. Reads (and caches)
    /// the calibration data first if needed.
    pub fn measure(&mut self, request: &SweepRequest) -> Result<Vec<Measurement>> {
        let registers = request.registers(self.calibration()?.xtal_freq_hz)?;
        let channel = self.channel.as_mut();

        exec(channel, Command::InitParameter)?;
        exec_with_data(channel, Command::SetFStart, &registers.f_start_bytes())?;
        exec_with_data(channel, Command::SetFStop, &registers.f_stop_bytes())?;
        exec_with_data(channel, Command::SetFStep, &registers.f_step.to_be_bytes())?;
        exec_with_data(channel, Command::SetRbw, &[registers.rbw])?;
        exec_with_data(channel, Command::SetIf, &[registers.if_])?;
        exec_with_data(channel, Command::SetGain, &[registers.gain])?;
        let samples = get_spectrum(channel)?;

        let gains = self.calibration()?.frq_gains_tables[request.range_index()][request.ref_level_index()].gains;
        Ok(registers.measurements(&samples, &gains))
    }

    /// Gets the device calibration data.
    ///
    /// The data is read from flash on the first call and cached for subsequent calls.
    pub fn calibration(&mut self) -> Result<&Calibration> {
        if self.calibration.is_none() {
            self.calibration = Some(self.fetch_calibration()?);
        }

        Ok(self.calibration.as_ref().expect("calibration was just fetched"))
    }

    fn fetch_calibration(&mut self) -> Result<Calibration> {
        self.check_prog_header()?;
        let data = read_flash(self.channel.as_mut(), FLASH_CALIBRATION_ADDR, FLASH_CALIBRATION_SIZE)?;
        Calibration::try_from(data.as_slice())
    }

    fn check_prog_header(&mut self) -> Result<()> {
        let data = read_flash(self.channel.as_mut(), FLASH_PROG_HEADER_ADDR, FLASH_PROG_HEADER_SIZE)?;
        let prog_header = ProgHeader::try_from(data.as_slice())?;
        if prog_header.mem_type != FLASH_PROG_HEADER_TYPE {
            return Err(Error::InvalidData(format!(
                "invalid flash memory type, expected: 0x{:04X}, got: 0x{:04X}",
                FLASH_PROG_HEADER_TYPE, prog_header.mem_type
            )));
        }
        Ok(())
    }
}

/// Formats a version word as `major.minor` (high byte, low byte).
fn format_version(version: u16) -> String {
    format!("{}.{}", version >> 8, version & 0xFF)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    fn a_device_with(channel: MockChannel) -> Sa430 {
        Sa430::new(Box::new(channel))
    }

    #[test]
    fn given_a_device_when_core_version_then_format_as_major_minor() {
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::GetCoreVersion));
        channel.add_response(&a_get_core_version_response());

        assert_eq!(a_device_with(channel).core_version().unwrap(), "2.10");
    }

    #[test]
    fn given_a_device_when_calibration_then_decode_flash_data() {
        let mut channel = MockChannel::new();
        add_calibration_responses(&mut channel);
        let mut device = a_device_with(channel);

        let calibration = device.calibration().unwrap();

        assert_eq!(calibration.version(), "1.16");
        assert_eq!(calibration.date(), "Mo. Sep 19 2011");
        assert_eq!(calibration.hardware_id, 0x0908);
        assert_eq!(&calibration.serial_number, b"08FF41E50F8B3A34");
        assert_eq!(calibration.xtal_freq_hz, 25_999_588);
        assert_eq!(calibration.xtal_freq_error_ppm, 16);
    }

    #[test]
    fn given_a_negative_xtal_error_when_parse_calibration_then_decode_as_signed() {
        let mut data = [
            CALIBRATION_DATA_1,
            CALIBRATION_DATA_2,
            CALIBRATION_DATA_3,
            CALIBRATION_DATA_4,
            CALIBRATION_DATA_5,
            CALIBRATION_DATA_6,
            CALIBRATION_DATA_7,
        ]
        .concat();
        // xtal_freq_error_ppm sits at offset 97 of the calibration block.
        data[97..99].copy_from_slice(&(-16i16).to_be_bytes());

        let calibration = Calibration::try_from(data.as_slice()).unwrap();

        assert_eq!(calibration.xtal_freq_error_ppm, -16);
    }

    #[test]
    fn given_calibration_was_read_when_calibration_again_then_use_the_cache() {
        let mut channel = MockChannel::new();
        add_calibration_responses(&mut channel);
        let mut device = a_device_with(channel);

        let first = device.calibration().unwrap().clone();
        // No more responses are queued: a second flash read would time out.
        assert_eq!(device.calibration().unwrap(), &first);
    }

    #[test]
    fn given_a_wrong_prog_header_type_when_calibration_then_return_error() {
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::FlashRead));
        channel.add_response(&a_read_flash_response(&[0, 212, 212, 6, 63, 0, 0, 2, 2, 18]));

        let result = a_device_with(channel).calibration().map(|_| ());

        assert!(matches!(result, Err(Error::InvalidData(_))));
    }

    #[test]
    fn given_truncated_data_when_parse_calibration_then_return_error() {
        let result = Calibration::try_from(&[0u8; 100][..]);
        assert!(matches!(result, Err(Error::InvalidData(_))));
    }

    #[test]
    fn given_a_device_when_measure_then_configure_the_registers_and_apply_calibration() {
        let mut channel = MockChannel::new();
        add_calibration_responses(&mut channel);
        for command in [
            Command::InitParameter,
            Command::SetFStart,
            Command::SetFStop,
            Command::SetFStep,
            Command::SetRbw,
            Command::SetIf,
            Command::SetGain,
            Command::GetSpecNoInit,
        ] {
            channel.add_response(&an_ack_response(command));
        }
        channel.add_response(&a_spectrum_response(&[-80, 10]));
        channel.add_response(&an_error_response(0));
        let mut device = a_device_with(channel);
        let request = SweepRequest::new(433.0, 434.0, 0.1, Some(-40)).unwrap();

        let measurements = device.measure(&request).unwrap();

        let gains = device.calibration().unwrap().frq_gains_tables[1][1].gains;
        assert_eq!(measurements.len(), 2);
        for (measurement, sample) in measurements.iter().zip([-40.0, 5.0]) {
            let f_hz = measurement.frequency_mhz * 1e6;
            let beta: f64 = gains.iter().enumerate().map(|(i, a)| a * f_hz.powi(i as i32)).sum();
            assert!((measurement.power_dbm - (sample - beta)).abs() < 1e-6 * (1.0 + beta.abs()));
        }
        assert!((measurements[0].frequency_mhz - 433.0).abs() < 0.001);
    }

    #[test]
    fn given_invalid_calibration_when_measure_then_fail_before_configuring_the_device() {
        let mut device = a_device_with(MockChannel::new());
        let request = SweepRequest::new(433.0, 434.0, 0.1, None).unwrap();

        assert!(device.measure(&request).is_err());
    }
}
