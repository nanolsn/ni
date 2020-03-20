const MASK: u8 = 0b0000_0011;
const LONG_VALUE: u8 = 0b1000_0000;

pub fn decode_x(byte: u8) -> u8 {
    let x = byte & MASK;
    x
}

pub fn decode_xy(byte: u8) -> (u8, u8) {
    let x = byte & MASK;
    let y = byte >> 2 & MASK;
    (x, y)
}

pub fn decode_xyz(byte: u8) -> (u8, u8, u8) {
    let x = byte & MASK;
    let y = byte >> 2 & MASK;
    let z = byte >> 4 & MASK;
    (x, y, z)
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

pub fn decode_value<I>(bytes: &mut I) -> Option<u64>
    where
        I: Iterator<Item=u8>,
{
    let det = bytes.next()?;

    if det & LONG_VALUE == 0 {
        return Some(det as u64);
    }

    let mut bs: [u8; 8] = [0; 8];

    for i in 0..bytes_to_read(decode_x(det)) {
        bs[i] = bytes.next()?;
    }

    Some(u64::from_le_bytes(bs))
}
