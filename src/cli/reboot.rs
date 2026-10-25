use std::io;

use sa430::Sa430;

use super::CliResult;

pub fn reboot(device: &mut Sa430, output: &mut dyn io::Write) -> CliResult {
    writeln!(output, "Rebooting device...")?;
    device.reboot()?;
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
    fn given_a_channel_when_reboot_then_return_ok() {
        let mut output = Vec::new();
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::HardwareReset));

        let mut device = Sa430::new(Box::new(channel));

        reboot(&mut device, &mut output).unwrap();

        assert_eq!(output, b"Rebooting device...\nDone!\n");
    }
}
