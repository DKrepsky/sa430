use std::io;

use sa430::{Sa430, SweepRequest};

use super::CliResult;

/// Measures the spectrum and prints it as CSV: a header, then one `frequency_mhz,power_dbm` row per point.
pub fn capture(device: &mut Sa430, request: &SweepRequest, output: &mut dyn io::Write) -> CliResult {
    let measurements = device.measure(request)?;
    writeln!(output, "frequency_mhz,power_dbm")?;
    for measurement in measurements {
        writeln!(output, "{:.6},{:.2}", measurement.frequency_mhz, measurement.power_dbm)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use sa430::frame::Command;
    use sa430::test_utils::*;

    use super::*;

    fn a_device_that_measures(samples: &[i8]) -> Sa430 {
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
        channel.add_response(&a_spectrum_response(samples));
        channel.add_response(&an_error_response(0));
        Sa430::new(Box::new(channel))
    }

    #[test]
    fn given_a_measurement_when_capture_then_print_csv_with_frequency_and_power() {
        let mut device = a_device_that_measures(&[-80, 0, 10]);
        let request = SweepRequest::new(433.0, 434.0, 0.1, None).unwrap();
        let mut output = Vec::new();

        capture(&mut device, &request, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines[0], "frequency_mhz,power_dbm");
        assert_eq!(lines.len(), 4);
        for line in &lines[1..] {
            let columns: Vec<_> = line.split(',').collect();
            assert_eq!(columns.len(), 2);
            assert!(columns.iter().all(|c| c.parse::<f64>().is_ok()), "{}", line);
        }
        let first_frequency: f64 = lines[1].split(',').next().unwrap().parse().unwrap();
        assert!((first_frequency - 433.0).abs() < 0.001, "{}", lines[1]);
    }

    #[test]
    fn given_the_device_fails_when_capture_then_return_the_error_and_print_nothing() {
        let mut channel = MockChannel::new();
        add_calibration_responses(&mut channel);
        channel.add_response(&an_error_response(0x0324));
        let mut device = Sa430::new(Box::new(channel));
        let request = SweepRequest::new(433.0, 434.0, 0.1, None).unwrap();
        let mut output = Vec::new();

        let error = capture(&mut device, &request, &mut output).unwrap_err();

        assert!(error.to_string().contains("Unknown command (0x0324)"), "{}", error);
        assert!(output.is_empty());
    }
}
