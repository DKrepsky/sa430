use sa430::device::Sa430;

use std::{error, io};

pub fn blink(device: &mut Sa430, output: &mut dyn io::Write) -> Result<(), Box<dyn error::Error>> {
    writeln!(output, "Blinking LED...")?;
    device.blink()?;
    writeln!(output, "Done!")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use sa430::{
        channel::fixtures::MockChannel,
        frame::{fixture::an_ack_response, Command},
    };

    #[test]
    fn given_a_channel_when_blink_then_return_ok() {
        let mut output = Vec::new();
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::BlinkLed).to_bytes());

        let mut device = Sa430::new(Box::new(channel));

        blink(&mut device, &mut output).unwrap();

        assert_eq!(output, b"Blinking LED...\nDone!\n");
    }
}
