const SIZE_MASK: u8 = 0b0000_0011;
const LONG_VALUE: u8 = 0b1000_0000;
const CONST: u8 = 0b0100_0000;

pub fn is_long_value(byte: u8) -> bool { byte & LONG_VALUE != 0 }

pub fn is_const(byte: u8) -> bool { byte & CONST != 0 }

pub fn decode_x(byte: u8) -> u8 { byte & SIZE_MASK }

pub fn decode_xy(byte: u8) -> (u8, u8) { (decode_x(byte), byte >> 2 & SIZE_MASK) }

pub fn decode_xyz(byte: u8) -> (u8, u8, u8) {
    let (x, y) = decode_xy(byte);
    (x, y, byte >> 4 & SIZE_MASK)
}

pub fn bytes_to_read(n: u8) -> usize {
    match n {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        _ => panic!("Undefined number of bytes"),
    }
}

pub fn decode_u64<I>(bytes: &mut I, count: usize) -> Option<u64>
    where
        I: Iterator<Item=u8>,
{
    const U64_SIZE: usize = std::mem::size_of::<u64>();
    let mut bs: [u8; U64_SIZE] = [0; U64_SIZE];

    for i in 0..count {
        bs[i] = bytes.next()?;
    }

    Some(u64::from_le_bytes(bs))
}

pub fn decode<I>(bytes: &mut I) -> Option<u64>
    where
        I: Iterator<Item=u8>,
{
    let spec = bytes.next()?;

    if !is_long_value(spec) {
        return Some(spec as u64);
    }

    decode_u64(bytes, bytes_to_read(decode_x(spec)))
}

use super::operation::Value;

pub fn decode_value<I>(bytes: &mut I) -> Option<Value<u64>>
    where
        I: Iterator<Item=u8>,
{
    let spec = bytes.next()?;

    if !is_long_value(spec) {
        return Some(Value::Const(spec as u64));
    }

    let val = decode_u64(bytes, bytes_to_read(decode_x(spec)))?;

    if is_const(spec) {
        Some(Value::Const(val))
    } else {
        Some(Value::Ref(val as u32))
    }
}
