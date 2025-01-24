use std::{error, io};

use sa430::device::Sa430;

const DEFAULT_REF_LEVEL: i8 = -35;

pub struct CaptureParams {
    pub fstart: f64,
    pub fstop: f64,
    pub fstep: f64,
    pub ref_level: Option<i8>,
}

pub fn capture(_: &mut Sa430, params: &CaptureParams, output: &mut dyn io::Write) -> Result<(), Box<dyn error::Error>> {
    write!(
        output,
        "Capturing data from {:.2} MHz to {:.2} MHz with step of {:.2} MHz and a reference level of {} dBm...",
        params.fstart,
        params.fstop,
        params.fstep,
        params.ref_level.unwrap_or(DEFAULT_REF_LEVEL)
    )?;
    todo!("Implement capture command")
}
