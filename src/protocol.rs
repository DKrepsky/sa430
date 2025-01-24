//! # Protocol Module
//!
//! The `protocol` module provides a set of functions to facilitate communication with a device over a specified channel.
//! It includes utilities for sending commands, receiving responses, and handling data in various formats.
//! The module ensures that the communication protocol is adhered to, validating responses and parsing data as needed.
//!
//! It is designed to abstract the complexities of device communication, providing a simple interface for common
//! operations that return different types of data. It handles the low-level details of frame construction,
//! transmission, and validation, allowing users to focus on higher-level logic.
use std::{
    error::Error,
    io::{Read, Write},
};

use super::channel::*;
use super::frame::*;
use super::parser::*;

/// Sends a command to the device and returns the response as a string.
pub fn get_string(channel: &mut dyn Channel, command: Command) -> Result<String, Box<dyn Error>> {
    let result = exec_with_result(channel, command)?;
    let value = String::from_utf8(result)?;
    Ok(value)
}

/// Sends a command to the device and returns the response as a `u32`.
pub fn get_u32(channel: &mut dyn Channel, command: Command) -> Result<u32, Box<dyn Error>> {
    let result = exec_with_result(channel, command)?;
    let mut parser = ByteArrayParser::new(&result);
    parser.take_u32()
}

/// Sends a command to the device and returns the response as a `u16`.
pub fn get_u16(channel: &mut dyn Channel, command: Command) -> Result<u16, Box<dyn Error>> {
    let result = exec_with_result(channel, command)?;
    let mut parser = ByteArrayParser::new(&result);
    parser.take_u16()
}

/// Reads a block of data from the device's flash memory starting at the specified address and of the specified size.
pub fn read_flash(channel: &mut dyn Channel, addr: u16, size: u16) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut pointer = addr;
    let mut remains = size;
    let mut buffer = Vec::new();

    while remains > 0 {
        let chunk_size = if remains > 255 { 255 } else { remains };
        let data: Vec<u8> = [pointer.to_be_bytes(), chunk_size.to_be_bytes()].concat();
        let request = Frame::with_data(Command::FlashRead, &data);
        send_frame(&request, channel.writer())?;

        let ack = receive_frame(channel.reader())?;
        validate(&request, &ack)?;

        let response = receive_frame(channel.reader())?;
        validate(&request, &response)?;

        buffer.extend_from_slice(response.data());
        remains -= chunk_size;
        pointer += chunk_size;
    }

    Ok(buffer)
}

/// Executes a command and returns the response as a byte vector.
pub fn exec_with_result(channel: &mut dyn Channel, command: Command) -> Result<Vec<u8>, Box<dyn Error>> {
    let request = Frame::new(command);
    send_frame(&request, channel.writer())?;

    let ack = receive_frame(channel.reader())?;
    validate(&request, &ack)?;

    let response = receive_frame(channel.reader())?;
    validate(&request, &response)?;

    Ok(response.data().to_vec())
}

/// Validates the response frame against the request frame.
fn validate(request: &Frame, response: &Frame) -> Result<(), Box<dyn Error>> {
    if response.is_error() {
        let message = format!(
            "Error executing command {}: {} ({:04X})",
            request.cmd(),
            response.to_error_code().unwrap(),
            response.to_error_code().unwrap()
        );
        return Err(Box::from(message));
    }

    if request.cmd() != response.cmd() {
        let message = format!(
            "Invalid response, expected: {:?}, received: {:?}",
            request.cmd(),
            response.cmd()
        );
        return Err(Box::from(message));
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

fn send_frame(frame: &Frame, port: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    port.write_all(&frame.to_bytes())?;
    Ok(())
}

fn receive_frame(port: &mut dyn Read) -> Result<Frame, Box<dyn Error>> {
    let mut state = ReceiverState::Start;
    let mut buffer: [u8; 260] = [0; 260];
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

fn read(port: &mut dyn Read) -> Result<u8, Box<dyn Error>> {
    let mut byte = [0; 1];
    port.read_exact(byte.as_mut())?;
    Ok(byte[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::channel::fixtures::MockChannel;
    use crate::frame::fixture::*;

    #[test]
    fn given_a_channel_and_a_command_when_get_string_then_return_the_string() {
        let mut channel = MockChannel::new();

        channel.add_response(&an_ack_response(Command::GetIdn).to_bytes());
        channel.add_response(&a_get_idn_response().to_bytes());

        let result = get_string(&mut channel, Command::GetIdn).unwrap();

        assert_eq!(channel.write_buffer, Frame::new(Command::GetIdn).to_bytes());
        assert_eq!(result, "Texas Instruments,MSP-SA430-SUB1GHZ: RF Dev Support Tool,HW2.0");
    }

    #[test]
    fn given_a_channel_and_a_command_when_get_u32_then_return_the_value() {
        let mut channel: MockChannel = MockChannel::new();

        channel.add_response(&an_ack_response(Command::GetSerialNumber).to_bytes());
        channel.add_response(&a_get_serial_number_response().to_bytes());

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

        channel.add_response(&an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&a_read_flash_response(&data).to_bytes());

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

        channel.add_response(&an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&a_read_flash_response(&data_255).to_bytes());
        channel.add_response(&an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&a_read_flash_response(&data_255).to_bytes());
        channel.add_response(&an_ack_response(Command::FlashRead).to_bytes());
        channel.add_response(&a_read_flash_response(&data_155).to_bytes());

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
    fn given_a_frame_when_send_frame_then_send_frame_to_port() {
        let frame = Frame::with_data(Command::SetGain, &[0x00, 0x01]);
        let mut port = Vec::new();
        send_frame(&frame, &mut port).unwrap();
        assert_eq!(port, vec![0x2a, 0x02, 0x1B, 0x00, 0x01, 0x0F, 0xDC]);
    }

    #[test]
    fn given_a_frame_when_receive_frame_then_receive_frame_from_port() {
        let frame = Frame::with_data(Command::SetGain, &[0x00, 0x01]);
        let mut port = Vec::new();
        port.write_all(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_with_no_data_when_receive_frame_then_receive_frame_from_port() {
        let frame = Frame::with_data(Command::BlinkLed, &[]);
        let mut port = Vec::new();
        port.write_all(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_when_receive_frame_then_receive_frame_from_port_with_extra_bytes() {
        let frame = Frame::with_data(Command::SetGain, &[0x00, 0x01]);
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
        assert_eq!(
            result.err().unwrap().to_string(),
            "Invalid CRC, expected: 0x0001, current: 0x8528"
        );
    }
}
