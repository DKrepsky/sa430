/// Calculate CRC16 value for given slice of bytes
///
/// # Arguments
///
/// * `data` - A slice of bytes to calculate CRC16
///
/// # Returns
/// * `u16` - CRC16 value
pub fn crc16(raw: &[u8]) -> u16 {
    let mut crc = raw[0] as u16;

    for value in raw.iter().skip(1) {
        crc = crc.rotate_left(8);
        crc ^= *value as u16;
        crc ^= (crc & 0xff) >> 4;
        crc ^= (crc << 8) << 4;
        crc ^= ((crc & 0xff) << 4) << 1;
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_slice_when_crc16_then_compute_crc16_value() {
        let data = [0x2a, 0x00, 0x01];
        let crc = crc16(&data);
        assert_eq!(crc, 0x9509);
    }
}
