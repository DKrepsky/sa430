use sa430::device::Sa430;

/// Prints the device information to the output.
pub fn info(device: &mut Sa430, output: &mut dyn std::io::Write) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "IDN: {}", device.idn()?)?;
    writeln!(output, "Serial Number: {}", device.serial_number()?)?;
    writeln!(output, "Core Version: {}", device.core_version()?)?;
    writeln!(output, "Spectrum Version: {}", device.spectrum_version()?)?;
    writeln!(output, "Calibration Version: {}", device.calibration_version()?)?;
    writeln!(output, "Calibration Date: {}", device.calibration_date()?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use sa430::{
        channel::fixtures::MockChannel,
        frame::{self, Command},
    };

    #[test]
    fn given_a_device_when_info_then_print_device_info() {
        let mut output = Vec::new();

        let mut channel = MockChannel::new();
        channel.add_response(&frame::fixture::an_ack_response(Command::GetIdn).to_bytes());
        channel.add_response(&frame::fixture::a_get_idn_response().to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::GetSerialNumber).to_bytes());
        channel.add_response(&frame::fixture::a_get_serial_number_response().to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::GetCoreVersion).to_bytes());
        channel.add_response(&frame::fixture::a_get_core_version_response().to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::GetSpectrumVersion).to_bytes());
        channel.add_response(&frame::fixture::a_get_spectrum_version_response().to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::PROG_HEADER_DATA).to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::CALIBRATION_DATA_1).to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::CALIBRATION_DATA_2).to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::CALIBRATION_DATA_3).to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::CALIBRATION_DATA_4).to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::CALIBRATION_DATA_5).to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::CALIBRATION_DATA_6).to_bytes());
        channel.add_response(&frame::fixture::an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&frame::fixture::a_read_flash_response(frame::fixture::CALIBRATION_DATA_7).to_bytes());

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
                "Calibration Date: Mo. Sep 19 2011\0\n"
            )
        );
    }
}
