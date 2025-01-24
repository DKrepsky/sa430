use std::{error, io};

use sa430::device::Sa430;

pub fn reboot(device: &mut Sa430, output: &mut dyn io::Write) -> Result<(), Box<dyn error::Error>> {
    writeln!(output, "Rebooting device...")?;
    device.reboot()?;
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
    fn given_a_channel_when_reboot_then_return_ok() {
        let mut output = Vec::new();
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::HardwareReset).to_bytes());

        let mut device = Sa430::new(Box::new(channel));

        reboot(&mut device, &mut output).unwrap();

        assert_eq!(output, b"Rebooting device...\nDone!\n");
    }
}
