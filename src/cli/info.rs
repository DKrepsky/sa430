use std::io;

use sa430::Sa430;

use super::CliResult;

/// Prints the device information to the output.
pub fn info(device: &mut Sa430, output: &mut dyn io::Write) -> CliResult {
    writeln!(output, "IDN: {}", device.idn()?)?;
    writeln!(output, "Serial Number: {}", device.serial_number()?)?;
    writeln!(output, "Core Version: {}", device.core_version()?)?;
    writeln!(output, "Spectrum Version: {}", device.spectrum_version()?)?;
    let calibration = device.calibration()?;
    writeln!(output, "Calibration Version: {}", calibration.version())?;
    writeln!(output, "Calibration Date: {}", calibration.date())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use sa430::{frame::Command, test_utils::*};

    #[test]
    fn given_a_device_when_info_then_print_device_info() {
        let mut output = Vec::new();

        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::GetIdn));
        channel.add_response(&a_get_idn_response());
        channel.add_response(&an_ack_response(Command::GetSerialNumber));
        channel.add_response(&a_get_serial_number_response());
        channel.add_response(&an_ack_response(Command::GetCoreVersion));
        channel.add_response(&a_get_core_version_response());
        channel.add_response(&an_ack_response(Command::GetSpectrumVersion));
        channel.add_response(&a_get_spectrum_version_response());
        add_calibration_responses(&mut channel);

        let mut device = Sa430::new(Box::new(channel));

        info(&mut device, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(
            output,
            concat!(
                "IDN: Texas Instruments,MSP-SA430-SUB1GHZ: RF Dev Support Tool,HW2.0\n",
                "Serial Number: 2312\n",
                "Core Version: 2.10\n",
                "Spectrum Version: 2.5\n",
                "Calibration Version: 1.16\n",
                "Calibration Date: Mo. Sep 19 2011\n"
            )
        );
    }
}
