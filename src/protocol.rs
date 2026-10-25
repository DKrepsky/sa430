//! # Protocol Module
//!
//! Request/response exchange with the device on top of [`Frame`]s.
//!
//! Every command sent to the device is first answered with an **acknowledge** frame carrying the same command. Commands
//! that return data are then followed by a **response** frame, again carrying the same command. Either frame may be
//! replaced by an error frame ([`Command::GetLastError`]), which is turned into [`Error::Device`].
use std::io::{Read, Write};

use crate::channel::Channel;
use crate::error::{Error, Result};
use crate::frame::*;
use crate::parser::ByteArrayParser;

/// Sends a command to the device and returns the response as a string.
pub fn get_string(channel: &mut dyn Channel, command: Command) -> Result<String> {
    let result = exec_with_result(channel, command)?;
    String::from_utf8(result).map_err(|e| Error::InvalidData(format!("response is not valid UTF-8: {}", e)))
}

/// Sends a command to the device and returns the response as a `u32`.
pub fn get_u32(channel: &mut dyn Channel, command: Command) -> Result<u32> {
    let result = exec_with_result(channel, command)?;
    ByteArrayParser::new(&result).take_u32()
}

/// Sends a command to the device and returns the response as a `u16`.
pub fn get_u16(channel: &mut dyn Channel, command: Command) -> Result<u16> {
    let result = exec_with_result(channel, command)?;
    ByteArrayParser::new(&result).take_u16()
}

/// Reads `size` bytes of the device's flash memory starting at `addr`.
///
/// Requests are split into chunks of at most [`FRAME_MAX_DATA_SIZE`] bytes.
pub fn read_flash(channel: &mut dyn Channel, addr: u16, size: u16) -> Result<Vec<u8>> {
    let mut pointer = addr;
    let mut remains = size;
    let mut buffer = Vec::with_capacity(size as usize);

    while remains > 0 {
        let chunk_size = remains.min(FRAME_MAX_DATA_SIZE as u16);
        let data = [pointer.to_be_bytes(), chunk_size.to_be_bytes()].concat();
        let request = Frame::with_data(Command::FlashRead, &data)?;
        let response = transact(channel, &request, true)?;

        // Shorter responses are an error. Longer ones are tolerated (the final chunk may be padded) and truncated.
        let Some(chunk) = response.data().get(..chunk_size as usize) else {
            return Err(Error::InvalidData(format!(
                "flash read at 0x{:04X} returned {} bytes, expected {}",
                pointer,
                response.data().len(),
                chunk_size
            )));
        };

        buffer.extend_from_slice(chunk);
        remains -= chunk_size;
        pointer = pointer.wrapping_add(chunk_size);
    }

    Ok(buffer)
}

/// Executes a command that has no result.
pub fn exec(channel: &mut dyn Channel, command: Command) -> Result<()> {
    transact(channel, &Frame::new(command), false)?;
    Ok(())
}

/// Executes a command that carries `data` and has no result.
pub fn exec_with_data(channel: &mut dyn Channel, command: Command, data: &[u8]) -> Result<()> {
    transact(channel, &Frame::with_data(command, data)?, false)?;
    Ok(())
}

/// Executes a command and returns the response payload.
pub fn exec_with_result(channel: &mut dyn Channel, command: Command) -> Result<Vec<u8>> {
    let response = transact(channel, &Frame::new(command), true)?;
    Ok(response.data().to_vec())
}

/// Sends `request`, waits for its acknowledge and, if `has_response`, for its response frame (which is returned).
/// Otherwise the acknowledge frame is returned.
fn transact(channel: &mut dyn Channel, request: &Frame, has_response: bool) -> Result<Frame> {
    send_frame(request, channel)?;

    let ack = receive_frame(channel)?;
    validate(request, &ack)?;

    if !has_response {
        return Ok(ack);
    }

    let response = receive_frame(channel)?;
    validate(request, &response)?;
    Ok(response)
}

/// Validates the response frame against the request frame.
fn validate(request: &Frame, response: &Frame) -> Result<()> {
    if let Some(code) = response.to_error_code() {
        return Err(Error::Device {
            command: request.cmd(),
            code,
        });
    }

    if request.cmd() != response.cmd() {
        return Err(Error::UnexpectedResponse {
            expected: request.cmd(),
            received: response.cmd(),
        });
    }

    Ok(())
}

enum ReceiverState {
    Start,
    Length,
    Command,
    Data,
    CrcHigh,
    CrcLow,
}

fn send_frame(frame: &Frame, port: &mut dyn Write) -> Result<()> {
    port.write_all(&frame.to_bytes())?;
    port.flush()?;
    Ok(())
}

/// Reads bytes until a complete frame is received, skipping anything before the magic byte.
fn receive_frame(port: &mut dyn Read) -> Result<Frame> {
    let mut state = ReceiverState::Start;
    let mut buffer = [0u8; FRAME_MAX_SIZE];
    let mut index: usize = 0;

    loop {
        let byte = read(port)?;
        buffer[index] = byte;
        index += 1;

        match state {
            ReceiverState::Start => {
                if byte == FRAME_MAGIC_VALUE {
                    state = ReceiverState::Length;
                } else {
                    index = 0;
                }
            }
            ReceiverState::Length => {
                state = ReceiverState::Command;
            }
            ReceiverState::Command => {
                if buffer[FRAME_DATA_LENGTH_INDEX] > 0 {
                    state = ReceiverState::Data
                } else {
                    state = ReceiverState::CrcHigh;
                }
            }
            ReceiverState::Data => {
                if index == FRAME_HEADER_SIZE + buffer[FRAME_DATA_LENGTH_INDEX] as usize {
                    state = ReceiverState::CrcHigh;
                }
            }
            ReceiverState::CrcHigh => {
                state = ReceiverState::CrcLow;
            }
            ReceiverState::CrcLow => {
                break;
            }
        }
    }

    let frame = Frame::from_bytes(&buffer[..index])?;
    Ok(frame)
}

fn read(port: &mut dyn Read) -> Result<u8> {
    let mut byte = [0; 1];
    port.read_exact(byte.as_mut())?;
    Ok(byte[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::*;

    #[test]
    fn given_a_channel_and_a_command_when_get_string_then_return_the_string() {
        let mut channel = MockChannel::new();

        channel.add_response(&an_ack_response(Command::GetIdn));
        channel.add_response(&a_get_idn_response());

        let result = get_string(&mut channel, Command::GetIdn).unwrap();

        assert_eq!(channel.write_buffer, Frame::new(Command::GetIdn).to_bytes());
        assert_eq!(result, "Texas Instruments,MSP-SA430-SUB1GHZ: RF Dev Support Tool,HW2.0");
    }

    #[test]
    fn given_a_channel_and_a_command_when_get_u32_then_return_the_value() {
        let mut channel: MockChannel = MockChannel::new();

        channel.add_response(&an_ack_response(Command::GetSerialNumber));
        channel.add_response(&a_get_serial_number_response());

        let result = get_u32(&mut channel, Command::GetSerialNumber).unwrap();

        assert_eq!(channel.write_buffer, Frame::new(Command::GetSerialNumber).to_bytes());
        assert_eq!(result, 0x0908);
    }

    #[test]
    fn given_an_address_and_a_size_smaller_than_255_when_read_flash_then_return_bytes_from_flash() {
        let addr: u16 = 0x4321;
        let size: u16 = 0x0044;
        let data = vec![0x01; size as usize];
        let mut channel = MockChannel::new();

        channel.add_response(&an_ack_response(Command::FlashRead));
        channel.add_response(&a_read_flash_response(&data));

        let result = read_flash(&mut channel, addr, size).unwrap();

        assert_eq!(
            channel.write_buffer,
            vec![0x2A, 0x04, 0x0A, 0x43, 0x21, 0x00, 0x44, 0x42, 0xE3]
        );
        assert_eq!(result.len(), size as usize);
        assert_eq!(result, a_read_flash_response(&data).data().to_vec());
    }

    #[test]
    fn given_an_address_and_a_size_greater_than_255_when_read_flash_then_return_bytes_from_flash() {
        let addr: u16 = 0x4321;
        let size: u16 = 0x0299; // 665 = 2 * 255 + 155 = 3 packets
        let data_255 = vec![0x01; 255];
        let data_155 = vec![0x01; 155];
        let mut channel = MockChannel::new();

        channel.add_response(&an_ack_response(Command::FlashRead));
        channel.add_response(&a_read_flash_response(&data_255));
        channel.add_response(&an_ack_response(Command::FlashRead));
        channel.add_response(&a_read_flash_response(&data_255));
        channel.add_response(&an_ack_response(Command::FlashRead));
        channel.add_response(&a_read_flash_response(&data_155));

        let result = read_flash(&mut channel, addr, size).unwrap();

        assert_eq!(
            channel.write_buffer,
            vec![
                42, 4, 10, 67, 33, 0, 255, 84, 83, 42, 4, 10, 68, 32, 0, 255, 50, 78, 42, 4, 10, 69, 31, 0, 155, 129,
                76
            ]
        );
        assert_eq!(result.len(), size as usize);
    }

    #[test]
    fn given_a_command_and_data_when_exec_with_data_then_send_the_data_and_wait_for_ack() {
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::HardwareReset));

        exec_with_data(&mut channel, Command::HardwareReset, &[0xDE, 0xAD]).unwrap();

        assert_eq!(channel.write_buffer, vec![0x2A, 0x02, 0x03, 0xDE, 0xAD, 0xA7, 0x50]);
    }

    #[test]
    fn given_a_frame_when_send_frame_then_send_frame_to_port() {
        let frame = Frame::with_data(Command::SetGain, &[0x00, 0x01]).unwrap();
        let mut port = Vec::new();
        send_frame(&frame, &mut port).unwrap();
        assert_eq!(port, vec![0x2a, 0x02, 0x1B, 0x00, 0x01, 0x0F, 0xDC]);
    }

    #[test]
    fn given_a_frame_when_receive_frame_then_receive_frame_from_port() {
        let frame = Frame::with_data(Command::SetGain, &[0x00, 0x01]).unwrap();
        let mut port = Vec::new();
        port.write_all(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_with_no_data_when_receive_frame_then_receive_frame_from_port() {
        let frame = Frame::new(Command::BlinkLed);
        let mut port = Vec::new();
        port.write_all(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_when_receive_frame_then_receive_frame_from_port_with_extra_bytes() {
        let frame = Frame::with_data(Command::SetGain, &[0x00, 0x01]).unwrap();
        let mut port = Vec::new();
        port.write_all(&[0x00, 0x00, 0x00]).unwrap();
        port.write_all(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_with_error_when_receive_frame_then_return_error() {
        let port = vec![0x2A, 0x00, 0x00, 0x00, 0x01];
        let result = receive_frame(&mut port.as_slice());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::Frame(FrameError::InvalidCrc(0x0001, 0x8528))
        ));
    }

    #[test]
    fn given_an_error_frame_instead_of_ack_when_exec_then_return_device_error() {
        let mut channel = MockChannel::new();
        channel.add_response(&an_error_response(ErrorCode::UnknownCmd as u16));

        let result = exec(&mut channel, Command::BlinkLed);

        assert!(matches!(
            result.unwrap_err(),
            Error::Device {
                command: Command::BlinkLed,
                code: ErrorCode::UnknownCmd
            }
        ));
    }

    #[test]
    fn given_an_ack_for_another_command_when_exec_then_return_unexpected_response() {
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::GetIdn));

        let result = exec(&mut channel, Command::BlinkLed);

        assert!(matches!(
            result.unwrap_err(),
            Error::UnexpectedResponse {
                expected: Command::BlinkLed,
                received: Command::GetIdn
            }
        ));
    }

    #[test]
    fn given_no_response_when_exec_then_return_io_error() {
        let mut channel = MockChannel::new();

        let result = exec(&mut channel, Command::BlinkLed);

        assert!(matches!(result.unwrap_err(), Error::Io(e) if e.kind() == std::io::ErrorKind::TimedOut));
    }

    #[test]
    fn given_a_short_flash_read_response_when_read_flash_then_return_error() {
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::FlashRead));
        channel.add_response(&a_read_flash_response(&[0x01; 3]));

        let result = read_flash(&mut channel, 0x1000, 4);

        assert!(matches!(result.unwrap_err(), Error::InvalidData(_)));
    }

    #[test]
    fn given_a_padded_flash_read_response_when_read_flash_then_truncate_to_requested_size() {
        let mut channel = MockChannel::new();
        channel.add_response(&an_ack_response(Command::FlashRead));
        channel.add_response(&a_read_flash_response(&[0x01, 0x02, 0xFF, 0xFF]));

        let result = read_flash(&mut channel, 0x1000, 2).unwrap();

        assert_eq!(result, vec![0x01, 0x02]);
    }
}
