use std::{
    error::Error,
    io::{Read, Write},
};

use super::frame::*;

enum ReceiverState {
    Start,
    Length,
    Command,
    Data,
    CrcHigh,
    CrcLow,
}

pub fn send_frame(frame: &Frame, port: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    port.write(&frame.to_bytes())?;
    Ok(())
}

pub fn receive_frame(port: &mut dyn Read) -> Result<Frame, Box<dyn Error>> {
    let mut state = ReceiverState::Start;
    let mut buffer: [u8; 256] = [0; 256];
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

    #[test]
    fn given_a_frame_when_send_frame_then_send_frame_to_port() {
        let frame = Frame::new(Command::SetGain, vec![0x00, 0x01]);
        let mut port = Vec::new();
        send_frame(&frame, &mut port).unwrap();
        assert_eq!(port, vec![0x2a, 0x02, 0x1B, 0x00, 0x01, 0x0F, 0xDC]);
    }

    #[test]
    fn given_a_frame_when_receive_frame_then_receive_frame_from_port() {
        let frame = Frame::new(Command::SetGain, vec![0x00, 0x01]);
        let mut port = Vec::new();
        port.write(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_with_no_data_when_receive_frame_then_receive_frame_from_port() {
        let frame = Frame::new(Command::BlinkLed, vec![]);
        let mut port = Vec::new();
        port.write(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_when_receive_frame_then_receive_frame_from_port_with_extra_bytes() {
        let frame = Frame::new(Command::SetGain, vec![0x00, 0x01]);
        let mut port = Vec::new();
        port.write(&[0x00, 0x00, 0x00]).unwrap();
        port.write(&frame.to_bytes()).unwrap();
        let received_frame = receive_frame(&mut port.as_slice()).unwrap();
        assert_eq!(frame, received_frame);
    }

    #[test]
    fn given_a_frame_with_error_when_receive_frame_then_receive_frame_from_port() {
        let port = vec![0x2A, 0x00, 0x00, 0x00, 0x01];
        let result = receive_frame(&mut port.as_slice());
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Invalid CRC, expected: 0x0001, current: 0x8528"
        );
    }
}
