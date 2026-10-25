//! # Spectrum Module
//!
//! Validation of measurement parameters, conversion to device register values and decoding of the raw samples into
//! [`Measurement`]s. See `docs/protocol.md` ("RF settings" and "Collecting measurements") for the formulas.
use crate::error::{Error, Result};

/// Reference level used when none is requested, in dBm.
pub const DEFAULT_REF_LEVEL_DBM: i8 = -35;

/// Reference levels supported by the device, in dBm.
pub const REF_LEVELS_DBM: [i8; 8] = [-35, -40, -45, -50, -55, -60, -65, -70];

/// Gain register value for each of [`REF_LEVELS_DBM`].
const REF_LEVEL_REGISTERS: [u8; 8] = [128, 144, 145, 74, 12, 179, 44, 61];

/// Frequency ranges supported by the device, in MHz.
pub const FREQUENCY_RANGES_MHZ: [(f64, f64); 3] = [(300.0, 348.0), (389.0, 464.0), (779.0, 928.0)];

/// Smallest step frequency, in MHz (1 kHz).
pub const MIN_STEP_MHZ: f64 = 0.001;

/// Largest step frequency, in MHz.
pub const MAX_STEP_MHZ: f64 = 75.0;

/// Resolution bandwidth table: (RBW in kHz, RBW register, IF register).
const RBW_TABLE: [(f64, u8, u8); 16] = [
    (58.0, 240, 8),
    (67.7, 224, 7),
    (81.3, 208, 7),
    (101.6, 192, 8),
    (116.1, 176, 7),
    (135.4, 160, 7),
    (162.5, 144, 8),
    (203.1, 128, 8),
    (232.1, 112, 8),
    (270.8, 96, 10),
    (325.0, 80, 11),
    (406.3, 64, 10),
    (464.3, 48, 12),
    (541.7, 32, 13),
    (650.0, 16, 16),
    (812.5, 0, 18),
];

/// A single point of the spectrum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    /// Frequency in MHz.
    pub frequency_mhz: f64,
    /// Power in dBm.
    pub power_dbm: f64,
}

/// A validated request for a spectrum measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SweepRequest {
    start_mhz: f64,
    stop_mhz: f64,
    step_mhz: f64,
    ref_level_dbm: i8,
    range_index: usize,
    ref_level_index: usize,
}

impl SweepRequest {
    /// Validates the parameters. Frequencies are in MHz; `ref_level_dbm` defaults to [`DEFAULT_REF_LEVEL_DBM`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if a frequency is outside the supported ranges, start and stop are not in
    /// the same range, the step is out of range, or the reference level is not one of [`REF_LEVELS_DBM`].
    pub fn new(start_mhz: f64, stop_mhz: f64, step_mhz: f64, ref_level_dbm: Option<i8>) -> Result<Self> {
        let start_range = range_of(start_mhz, "start")?;
        let stop_range = range_of(stop_mhz, "stop")?;
        if start_range != stop_range {
            return Err(Error::InvalidParameter(
                "start and stop frequencies must be in the same frequency range".into(),
            ));
        }
        if start_mhz >= stop_mhz {
            return Err(Error::InvalidParameter(
                "start frequency must be lower than stop frequency".into(),
            ));
        }
        if !(MIN_STEP_MHZ..=MAX_STEP_MHZ).contains(&step_mhz) {
            return Err(Error::InvalidParameter(format!(
                "step frequency {} MHz is out of range, it must be between {} and {} MHz",
                step_mhz, MIN_STEP_MHZ, MAX_STEP_MHZ
            )));
        }
        let ref_level_dbm = ref_level_dbm.unwrap_or(DEFAULT_REF_LEVEL_DBM);
        let Some(ref_level_index) = REF_LEVELS_DBM.iter().position(|&level| level == ref_level_dbm) else {
            return Err(Error::InvalidParameter(format!(
                "reference level {} dBm is not valid, it must be one of {:?}",
                ref_level_dbm, REF_LEVELS_DBM
            )));
        };

        Ok(Self {
            start_mhz,
            stop_mhz,
            step_mhz,
            ref_level_dbm,
            range_index: start_range,
            ref_level_index,
        })
    }

    /// Start frequency in MHz.
    pub fn start_mhz(&self) -> f64 {
        self.start_mhz
    }

    /// Stop frequency in MHz.
    pub fn stop_mhz(&self) -> f64 {
        self.stop_mhz
    }

    /// Step frequency in MHz.
    pub fn step_mhz(&self) -> f64 {
        self.step_mhz
    }

    /// Reference level in dBm.
    pub fn ref_level_dbm(&self) -> i8 {
        self.ref_level_dbm
    }

    /// Index (0 to 2) of the frequency range containing the sweep.
    pub(crate) fn range_index(&self) -> usize {
        self.range_index
    }

    /// Index of the reference level in [`REF_LEVELS_DBM`].
    pub(crate) fn ref_level_index(&self) -> usize {
        self.ref_level_index
    }

    /// Computes the device registers, compensating the frequencies for the crystal frequency `xtal_hz`.
    pub(crate) fn registers(&self, xtal_hz: u32) -> Result<Registers> {
        let xtal_mhz = xtal_hz as f64 / 1e6;
        let compensate = |freq_mhz: f64| (freq_mhz * 65536.0 / xtal_mhz) as u32;

        let f_start = compensate(self.start_mhz) & FREQUENCY_MASK;
        let f_stop = compensate(self.stop_mhz) & FREQUENCY_MASK;
        let f_step = u16::try_from(compensate(self.step_mhz)).map_err(|_| {
            Error::InvalidParameter(format!(
                "step frequency {} MHz is too large for this device",
                self.step_mhz
            ))
        })?;

        // The RBW must be at least twice the step. Use the widest filter when even that is not enough.
        let step_khz = xtal_hz as f64 * f_step as f64 / 65536.0 / 1e3;
        let &(_, rbw, if_) = RBW_TABLE
            .iter()
            .find(|(rbw_khz, _, _)| *rbw_khz >= 2.0 * step_khz)
            .unwrap_or(&RBW_TABLE[RBW_TABLE.len() - 1]);

        Ok(Registers {
            f_start,
            f_stop,
            f_step,
            rbw,
            if_,
            gain: REF_LEVEL_REGISTERS[self.ref_level_index],
            xtal_hz,
        })
    }
}

/// Only the lower 24 bits of the start and stop frequency registers are sent.
const FREQUENCY_MASK: u32 = 0x00FF_FFFF;

fn range_of(freq_mhz: f64, name: &str) -> Result<usize> {
    FREQUENCY_RANGES_MHZ
        .iter()
        .position(|(low, high)| (*low..=*high).contains(&freq_mhz))
        .ok_or_else(|| {
            Error::InvalidParameter(format!(
                "{} frequency {} MHz is out of range, supported ranges are 300-348, 389-464 and 779-928 MHz",
                name, freq_mhz
            ))
        })
}

/// Register values describing a sweep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Registers {
    pub f_start: u32,
    pub f_stop: u32,
    pub f_step: u16,
    pub rbw: u8,
    pub if_: u8,
    pub gain: u8,
    xtal_hz: u32,
}

impl Registers {
    /// Start register as sent to the device (3 bytes, big-endian).
    pub fn f_start_bytes(&self) -> [u8; 3] {
        let [_, a, b, c] = self.f_start.to_be_bytes();
        [a, b, c]
    }

    /// Stop register as sent to the device (3 bytes, big-endian).
    pub fn f_stop_bytes(&self) -> [u8; 3] {
        let [_, a, b, c] = self.f_stop.to_be_bytes();
        [a, b, c]
    }

    /// Converts the raw samples into measurements, using the gain polynomial `gains` of the selected range and
    /// reference level.
    ///
    /// The frequencies are the ones the device really uses, which are quantized by the crystal frequency.
    pub fn measurements(&self, samples: &[u8], gains: &[f64; 8]) -> Vec<Measurement> {
        let hz_per_lsb = self.xtal_hz as f64 / 65536.0;
        let start_hz = hz_per_lsb * self.f_start as f64;
        let step_hz = hz_per_lsb * self.f_step as f64;

        samples
            .iter()
            .enumerate()
            .map(|(n, &sample)| {
                let frequency_hz = start_hz + n as f64 * step_hz;
                let beta = gains.iter().rev().fold(0.0, |acc, alpha| acc * frequency_hz + alpha);
                Measurement {
                    frequency_mhz: frequency_hz / 1e6,
                    power_dbm: (sample as i8) as f64 / 2.0 - beta,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XTAL_HZ: u32 = 26_000_000;

    fn request(start: f64, stop: f64, step: f64, ref_level: Option<i8>) -> Result<SweepRequest> {
        SweepRequest::new(start, stop, step, ref_level)
    }

    #[test]
    fn given_valid_parameters_when_new_then_use_default_reference_level() {
        let request = request(433.0, 434.0, 0.1, None).unwrap();

        assert_eq!(request.ref_level_dbm(), -35);
        assert_eq!(request.range_index(), 1);
        assert_eq!(request.ref_level_index(), 0);
    }

    #[test]
    fn given_every_supported_reference_level_when_new_then_ok() {
        for level in REF_LEVELS_DBM {
            assert!(request(433.0, 434.0, 0.1, Some(level)).is_ok());
        }
    }

    #[test]
    fn given_a_range_edge_when_new_then_ok() {
        assert!(request(300.0, 348.0, 1.0, None).is_ok());
        assert!(request(779.0, 928.0, 75.0, None).is_ok());
        assert!(request(433.0, 434.0, 0.001, None).is_ok());
    }

    #[test]
    fn given_an_out_of_range_frequency_when_new_then_error() {
        for (start, stop) in [
            (299.9, 310.0),
            (310.0, 348.1),
            (350.0, 360.0),
            (900.0, 929.0),
            (f64::NAN, 434.0),
        ] {
            let error = request(start, stop, 0.1, None).unwrap_err();
            assert!(error.to_string().contains("out of range"), "{}", error);
        }
    }

    #[test]
    fn given_start_and_stop_in_different_ranges_when_new_then_error() {
        let error = request(340.0, 400.0, 0.1, None).unwrap_err();
        assert!(error.to_string().contains("same frequency range"));
    }

    #[test]
    fn given_start_not_below_stop_when_new_then_error() {
        assert!(request(434.0, 433.0, 0.1, None).is_err());
        assert!(request(433.0, 433.0, 0.1, None).is_err());
    }

    #[test]
    fn given_an_out_of_range_step_when_new_then_error() {
        for step in [0.0, 0.0009, 75.1, -1.0, f64::NAN] {
            let error = request(433.0, 434.0, step, None).unwrap_err();
            assert!(error.to_string().contains("out of range"), "{}", error);
        }
    }

    #[test]
    fn given_an_invalid_reference_level_when_new_then_error() {
        for level in [-34, -36, 0, -75] {
            let error = request(433.0, 434.0, 0.1, Some(level)).unwrap_err();
            assert!(error.to_string().contains("reference level"), "{}", error);
        }
    }

    #[test]
    fn given_a_request_when_registers_then_compensate_frequencies() {
        let registers = request(433.0, 434.0, 0.1, Some(-40))
            .unwrap()
            .registers(XTAL_HZ)
            .unwrap();

        assert_eq!(registers.f_start, (433.0_f64 * 65536.0 / 26.0) as u32);
        assert_eq!(registers.f_stop, (434.0_f64 * 65536.0 / 26.0) as u32);
        assert_eq!(registers.f_step, (0.1_f64 * 65536.0 / 26.0) as u16);
        assert_eq!(registers.gain, 144);
    }

    #[test]
    fn given_a_small_step_when_registers_then_use_the_narrowest_filter() {
        let registers = request(433.0, 434.0, 0.01, None).unwrap().registers(XTAL_HZ).unwrap();

        assert_eq!((registers.rbw, registers.if_), (240, 8));
    }

    #[test]
    fn given_a_step_when_registers_then_rbw_is_at_least_twice_the_step() {
        // 0.1 MHz step needs an RBW of at least 200 kHz: 203.1 kHz.
        let registers = request(433.0, 434.0, 0.1, None).unwrap().registers(XTAL_HZ).unwrap();

        assert_eq!((registers.rbw, registers.if_), (128, 8));
    }

    #[test]
    fn given_a_step_wider_than_any_filter_when_registers_then_use_the_widest_filter() {
        let registers = request(779.0, 928.0, 10.0, None).unwrap().registers(XTAL_HZ).unwrap();

        assert_eq!((registers.rbw, registers.if_), (0, 18));
    }

    #[test]
    fn given_a_step_that_overflows_its_register_when_registers_then_error() {
        let request = request(779.0, 928.0, 75.0, None).unwrap();

        assert!(matches!(request.registers(XTAL_HZ), Err(Error::InvalidParameter(_))));
    }

    #[test]
    fn given_a_frequency_when_register_bytes_then_big_endian_24_bits() {
        let registers = request(433.0, 434.0, 0.1, None).unwrap().registers(XTAL_HZ).unwrap();

        assert_eq!(registers.f_start_bytes(), [0x10, 0xA7, 0x62][..]);
        assert_eq!(registers.f_stop_bytes(), [0x10, 0xB1, 0x3B][..]);
    }

    #[test]
    fn given_samples_when_measurements_then_apply_the_offset_and_compute_frequencies() {
        let registers = request(433.0, 434.0, 0.1, None).unwrap().registers(XTAL_HZ).unwrap();
        let step_mhz = XTAL_HZ as f64 * registers.f_step as f64 / 65536.0 / 1e6;
        let gains = [3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let measurements = registers.measurements(&[(-80_i8) as u8, 0, 10], &gains);

        assert_eq!(measurements.len(), 3);
        assert!((measurements[0].frequency_mhz - 433.0).abs() < 0.001);
        assert!((measurements[1].frequency_mhz - measurements[0].frequency_mhz - step_mhz).abs() < 1e-9);
        assert_eq!(measurements[0].power_dbm, -43.0);
        assert_eq!(measurements[1].power_dbm, -3.0);
        assert_eq!(measurements[2].power_dbm, 2.0);
    }

    #[test]
    fn given_a_polynomial_when_measurements_then_evaluate_it_at_the_frequency_in_hz() {
        let registers = request(433.0, 434.0, 0.1, None).unwrap().registers(XTAL_HZ).unwrap();
        let gains = [1.0, 2e-9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let measurement = registers.measurements(&[0], &gains)[0];

        let f_hz = measurement.frequency_mhz * 1e6;
        assert!((measurement.power_dbm + (1.0 + 2e-9 * f_hz)).abs() < 1e-6);
    }
}
