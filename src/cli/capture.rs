use std::io;

use sa430::Sa430;

use super::CliResult;

const DEFAULT_REF_LEVEL: i8 = -35;

/// Reference levels supported by the device, in dBm.
const REF_LEVELS: [i8; 8] = [-35, -40, -45, -50, -55, -60, -65, -70];

pub struct CaptureParams {
    pub fstart: f64,
    pub fstop: f64,
    pub fstep: f64,
    pub ref_level: Option<i8>,
}

impl CaptureParams {
    /// Checks the parameters are consistent before talking to the device.
    pub fn validate(&self) -> CliResult {
        if self.fstart >= self.fstop {
            return Err("fstart must be lower than fstop".into());
        }
        if self.fstep <= 0.0 {
            return Err("fstep must be greater than zero".into());
        }
        if !REF_LEVELS.contains(&self.ref_level()) {
            return Err(format!("rlevel must be one of {:?} dBm", REF_LEVELS).into());
        }
        Ok(())
    }

    pub fn ref_level(&self) -> i8 {
        self.ref_level.unwrap_or(DEFAULT_REF_LEVEL)
    }
}

pub fn capture(_: &mut Sa430, params: &CaptureParams, output: &mut dyn io::Write) -> CliResult {
    writeln!(
        output,
        "Capturing data from {:.2} MHz to {:.2} MHz with step of {:.2} MHz and a reference level of {} dBm...",
        params.fstart,
        params.fstop,
        params.fstep,
        params.ref_level()
    )?;
    // TODO(#6): implement spectrum capture.
    Err("capture is not implemented yet".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(fstart: f64, fstop: f64, fstep: f64, ref_level: Option<i8>) -> CaptureParams {
        CaptureParams {
            fstart,
            fstop,
            fstep,
            ref_level,
        }
    }

    #[test]
    fn given_valid_params_when_validate_then_ok() {
        assert!(params(433.0, 434.0, 0.1, None).validate().is_ok());
        assert!(params(433.0, 434.0, 0.1, Some(-70)).validate().is_ok());
    }

    #[test]
    fn given_invalid_params_when_validate_then_error() {
        assert!(params(434.0, 433.0, 0.1, None).validate().is_err());
        assert!(params(433.0, 434.0, 0.0, None).validate().is_err());
        assert!(params(433.0, 434.0, 0.1, Some(-36)).validate().is_err());
    }
}
