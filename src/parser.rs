/// A parser for reading various types of data from a byte array.
///
/// # Examples
///
/// ```rust
/// use sa430::parser::ByteArrayParser;
///
/// let data = vec![0x01, 0x02, 0x03, 0x04];
/// let mut parser = ByteArrayParser::new(&data);
///
/// assert_eq!(parser.take_u8().unwrap(), 0x01);
/// assert_eq!(parser.take_u16().unwrap(), 0x0203);
/// assert_eq!(parser.take_u32().unwrap_err().to_string(), "index out of bounds: the len is 4 but the index is 4");
/// ```
///
/// # Errors
///
/// Each method returns a `Result` which will contain an error if the buffer does not have enough data to fulfill the request.
use std::error::Error;

/// A parser for reading various types of data from a byte buffer.
pub struct ByteArrayParser<'a> {
    offset: usize,
    buffer: &'a [u8],
}

impl ByteArrayParser<'_> {
    /// Creates a new `ByteArrayParser` with the given slice.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A slice of bytes to be parsed.
    pub fn new(buffer: &[u8]) -> ByteArrayParser {
        ByteArrayParser { offset: 0, buffer }
    }

    /// Takes a single byte from the buffer.
    ///
    /// # Returns
    ///
    /// A `Result` containing the byte or an error if the buffer does not have enough data.
    pub fn take_u8(&mut self) -> Result<u8, Box<dyn Error>> {
        if self.offset >= self.buffer.len() {
            return Err("index out of bounds".into());
        }

        let value = u8::from_be_bytes([self.buffer[self.offset]]);
        self.offset += 1;
        Ok(value)
    }

    /// Takes two bytes from the buffer and interprets them as a big-endian `u16`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `u16` or an error if the buffer does not have enough data.
    pub fn take_u16(&mut self) -> Result<u16, Box<dyn Error>> {
        if self.offset + 2 >= self.buffer.len() {
            return Err("index out of bounds".into());
        }

        let value = u16::from_be_bytes(self.buffer[self.offset..self.offset + 2].try_into()?);
        self.offset += 2;
        Ok(value)
    }

    /// Takes four bytes from the buffer and interprets them as a big-endian `u32`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `u32` or an error if the buffer does not have enough data.
    pub fn take_u32(&mut self) -> Result<u32, Box<dyn Error>> {
        if self.offset + 4 >= self.buffer.len() {
            return Err("index out of bounds".into());
        }

        let value = u32::from_be_bytes(self.buffer[self.offset..self.offset + 4].try_into()?);
        self.offset += 4;
        Ok(value)
    }

    /// Takes a specified number of bytes from the buffer.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of bytes to take from the buffer.
    ///
    /// # Returns
    ///
    /// A `Result` containing a slice of the bytes or an error if the buffer does not have enough data.
    pub fn take_bytes(&mut self, size: usize) -> Result<&[u8], Box<dyn Error>> {
        if self.offset + size >= self.buffer.len() {
            return Err("index out of bounds".into());
        }

        let value = &self.buffer[self.offset..self.offset + size];
        self.offset += size;
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
    fn given_a_buffer_when_take_u32_then_return_the_u32() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut parser = ByteArrayParser::new(&data);

        assert_eq!(parser.take_u32().unwrap(), 0x01020304);
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

        assert_eq!(parser.take_bytes(5).unwrap_err().to_string(), "index out of bounds");
    }
}
