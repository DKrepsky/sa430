//! A cursor for reading big-endian (and the occasional little-endian) values from a byte slice.
//!
//! # Examples
//!
//! ```ignore
//! use sa430::parser::ByteArrayParser;
//!
//! let data = vec![0x01, 0x02, 0x03, 0x04];
//! let mut parser = ByteArrayParser::new(&data);
//!
//! assert_eq!(parser.take_u8().unwrap(), 0x01);
//! assert_eq!(parser.take_u16().unwrap(), 0x0203);
//! assert!(parser.take_u32().is_err());
//! ```
//!
//! # Errors
//!
//! Each method returns [`Error::InvalidData`] if the buffer does not have enough data to fulfill the request. The
//! offset is left untouched in that case.
use crate::error::{Error, Result};

/// A parser for reading various types of data from a byte buffer.
pub struct ByteArrayParser<'a> {
    offset: usize,
    buffer: &'a [u8],
}

impl<'a> ByteArrayParser<'a> {
    /// Creates a new `ByteArrayParser` over the given slice.
    pub fn new(buffer: &'a [u8]) -> Self {
        ByteArrayParser { offset: 0, buffer }
    }

    /// Takes a single byte.
    pub fn take_u8(&mut self) -> Result<u8> {
        Ok(self.take_array::<1>()?[0])
    }

    /// Takes two bytes and interprets them as a big-endian `u16`.
    pub fn take_u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.take_array()?))
    }

    /// Takes two bytes and interprets them as a little-endian `u16`.
    pub fn take_u16_le(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take_array()?))
    }

    /// Takes two bytes and interprets them as a big-endian `i16`.
    pub fn take_i16(&mut self) -> Result<i16> {
        Ok(i16::from_be_bytes(self.take_array()?))
    }

    /// Takes four bytes and interprets them as a big-endian `u32`.
    pub fn take_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.take_array()?))
    }

    /// Takes eight bytes and interprets them as a big-endian `f64`.
    pub fn take_f64(&mut self) -> Result<f64> {
        Ok(f64::from_be_bytes(self.take_array()?))
    }

    /// Takes exactly `N` bytes as an array.
    pub fn take_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let bytes = self.take_bytes(N)?;
        // Length was checked by `take_bytes`.
        Ok(bytes.try_into().expect("slice has N bytes"))
    }

    /// Takes `size` bytes as a slice.
    pub fn take_bytes(&mut self, size: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(size).filter(|end| *end <= self.buffer.len());
        let Some(end) = end else {
            return Err(Error::InvalidData(format!(
                "expected {} more bytes at offset {}, but only {} remain",
                size,
                self.offset,
                self.buffer.len() - self.offset
            )));
        };

        let value = &self.buffer[self.offset..end];
        self.offset = end;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_buffer_when_take_u8_then_return_the_byte() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_u8().unwrap(), 0x01);
    }

    #[test]
    fn given_a_buffer_when_take_u16_then_return_the_u16() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_u16().unwrap(), 0x0102);
    }

    #[test]
    fn given_a_buffer_when_take_u16_le_then_return_the_little_endian_u16() {
        let data = vec![0x01, 0x02];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_u16_le().unwrap(), 0x0201);
    }

    #[test]
    fn given_a_buffer_when_take_i16_then_return_the_signed_i16() {
        let data = vec![0xFF, 0xF0];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_i16().unwrap(), -16);
    }

    #[test]
    fn given_a_buffer_when_take_u32_then_return_the_u32() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_u32().unwrap(), 0x01020304);
    }

    #[test]
    fn given_a_buffer_when_take_f64_then_return_the_f64() {
        let data = 1.5f64.to_be_bytes();
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_f64().unwrap(), 1.5);
    }

    #[test]
    fn given_a_buffer_when_take_bytes_then_return_the_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_bytes(2).unwrap(), &[0x01, 0x02]);
    }

    #[test]
    fn given_a_buffer_when_take_bytes_then_advance_the_offset() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut parser = ByteArrayParser::new(&data);

        parser.take_bytes(2).unwrap();
        assert_eq!(parser.take_bytes(2).unwrap(), &[0x03, 0x04]);
    }

    #[test]
    fn given_a_buffer_when_take_bytes_then_return_an_error_if_not_enough_data() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(
            parser.take_bytes(5).unwrap_err().to_string(),
            "invalid data: expected 5 more bytes at offset 0, but only 4 remain"
        );
    }

    #[test]
    fn given_not_enough_data_when_take_then_offset_is_unchanged() {
        let data = vec![0x01, 0x02, 0x03];
        let mut parser = ByteArrayParser::new(&data);

        assert!(parser.take_u32().is_err());
        assert_eq!(parser.take_u16().unwrap(), 0x0102);
    }
}
