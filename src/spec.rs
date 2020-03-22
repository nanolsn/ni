use super::decode_utils::*;

#[derive(Copy, Clone, Debug)]
pub struct Spec(pub u8);

impl Spec {
    pub fn x(self) -> SpecField { SpecField(decode_x(self.0)) }
    pub fn y(self) -> SpecField { SpecField(decode_y(self.0)) }
    pub fn z(self) -> SpecField { SpecField(decode_z(self.0)) }
    pub fn w(self) -> SpecField { SpecField(decode_w(self.0)) }

    pub fn check_bits(self, mask: u8) -> bool { self.0 & mask != 0 }
}

impl From<u8> for Spec {
    fn from(byte: u8) -> Self { Spec(byte) }
}

#[derive(Copy, Clone, Debug)]
pub struct SpecField(pub u8);

impl SpecField {
    pub fn to_bits(self) -> u8 { self.0 }

    pub fn to_size(self) -> usize { to_size(self.0) }

    pub fn read_value<I>(self, bytes: &mut I) -> Option<u64>
        where
            I: Iterator<Item=u8>,
    { decode_u64(bytes, self.to_size()) }
}
