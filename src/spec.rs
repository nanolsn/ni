use super::decode_utils::*;

pub const CONST_BIT: u8 = 0b0100_0000;
pub const RETVAL_BIT: u8 = 0b1000_0000;

#[derive(Copy, Clone, Debug)]
pub struct SpecByte(pub u8);

impl SpecByte {
    pub fn x_size(self) -> usize { to_size(decode_x(self.0)) }
    pub fn y_size(self) -> usize { to_size(decode_y(self.0)) }
    pub fn z_size(self) -> usize { to_size(decode_z(self.0)) }
    pub fn w_size(self) -> usize { to_size(decode_w(self.0)) }

    pub fn check_bits(self, mask: u8) -> bool { self.0 & mask != 0 }
}

impl From<u8> for SpecByte {
    fn from(byte: u8) -> Self { SpecByte(byte) }
}

#[derive(Copy, Clone, Debug)]
pub struct SpecSet {
    x: usize,
    y: usize,
    op_size: usize,
    ret_value: bool,
    constant: bool,
}

impl SpecSet {
    pub fn new(byte: u8) -> Self {
        let spec = SpecByte(byte);

        SpecSet {
            x: spec.x_size(),
            y: spec.y_size(),
            op_size: spec.z_size(),
            ret_value: spec.check_bits(RETVAL_BIT),
            constant: spec.check_bits(CONST_BIT),
        }
    }
}
