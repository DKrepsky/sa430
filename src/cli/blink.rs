use std::io;

use sa430::Sa430;

use super::CliResult;

pub fn blink(device: &mut Sa430, output: &mut dyn io::Write) -> CliResult {
    writeln!(output, "Blinking LED...")?;
    device.blink()?;
    writeln!(output, "Done!")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use sa430::{
        frame::Command,
        test_utils::{an_ack_response, MockChannel},
    };

    #[test]
    fn given_a_channel_when_blink_then_return_ok() {
        let mut output = Vec::new();
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::BlinkLed));

        let mut device = Sa430::new(Box::new(channel));

        blink(&mut device, &mut output).unwrap();

        assert_eq!(output, b"Blinking LED...\nDone!\n");
    }

    #[test]
    fn given_the_device_does_not_answer_when_blink_then_return_error() {
        let mut output = Vec::new();
        let mut device = Sa430::new(Box::new(MockChannel::new()));

        assert!(blink(&mut device, &mut output).is_err());
    }
}
