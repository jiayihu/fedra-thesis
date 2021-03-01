use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

pub fn f32_to_u32(x: f32) -> u32 {
    unsafe { std::mem::transmute::<f32, u32>(x) }
}

pub fn f32_to_bytes(x: f32) -> Result<Vec<u8>> {
    let mut bytes = vec![];
    bytes.write_f32::<BigEndian>(x)?;

    Ok(bytes)
}

pub fn bytes_to_f32(bytes: Vec<u8>) -> Result<f32> {
    let mut rdr = Cursor::new(bytes);
    let value = rdr.read_f32::<BigEndian>()?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_to_bytes() {
        let value: f32 = 0.31375358;
        let mut bytes = vec![];
        bytes.write_f32::<BigEndian>(value);

        assert_eq!(bytes, Ok(vec![0x3E, 0xA0, 0xA4, 0x4F]));
    }

    #[test]
    fn test_bytes_to_f32() {
        let bytes = vec![0x3E, 0xA0, 0xA4, 0x4F];
        let value = bytes_to_f32(bytes);

        assert_eq!(value, Ok(0.31375358f32));
    }
}
