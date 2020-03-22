use super::decode_utils::*;
use super::operation::*;

const CONST_BIT: u8 = 0b0100_0000;
const NEXT_BIT: u8 = 0b1000_0000;
const RETVAL_BIT: u8 = 0b1000_0000;
const LONG_BIT: u8 = 0b1000_0000;

#[derive(Copy, Clone, Debug)]
pub struct Spec(pub u8);

impl Spec {
    fn field(self, field: u8) -> SpecField { SpecField { field, spec: self } }

    pub fn x(self) -> SpecField { self.field(decode_x(self.0)) }
    pub fn y(self) -> SpecField { self.field(decode_y(self.0)) }
    pub fn z(self) -> SpecField { self.field(decode_z(self.0)) }
    pub fn w(self) -> SpecField { self.field(decode_w(self.0)) }

    pub fn check_bits(self, mask: u8) -> bool { self.0 & mask != 0 }

    pub fn is_const(self) -> bool { self.check_bits(CONST_BIT) }
    pub fn is_next(self) -> bool { self.check_bits(NEXT_BIT) }
    pub fn is_retval(self) -> bool { self.check_bits(RETVAL_BIT) }
    pub fn is_long(self) -> bool { self.check_bits(LONG_BIT) }
}

impl From<u8> for Spec {
    fn from(byte: u8) -> Self { Spec(byte) }
}

#[derive(Copy, Clone, Debug)]
pub struct SpecField {
    pub field: u8,
    pub spec: Spec,
}

impl SpecField {
    pub fn to_bits(self) -> u8 { self.field }

    pub fn to_size(self) -> usize { to_size(self.field) }

    pub fn read<T, I>(self, bytes: &mut I) -> Option<T>
        where
            I: Iterator<Item=u8>,
            Self: Read<T>,
    { Read::<T>::read(self, bytes) }

    fn read_u64<I>(self, bytes: &mut I) -> Option<u64>
        where
            I: Iterator<Item=u8>,
    { decode_u64(bytes, self.to_size()) }

    fn read_value<I>(self, bytes: &mut I) -> Option<Value>
        where
            I: Iterator<Item=u8>,
    {
        let val = self.read_u64(bytes)?;

        if self.spec.is_const() {
            Some(Value::Const(val))
        } else {
            Some(Value::Ref(val as usize))
        }
    }

    fn read_retval<I>(self, bytes: &mut I) -> Option<RefRet>
        where
            I: Iterator<Item=u8>,
    {
        let val = self.read_u64(bytes)?;

        if self.spec.is_retval() {
            Some(RefRet::Return(val as usize))
        } else {
            Some(RefRet::Ref(val as usize))
        }
    }

    fn read_ref<I>(self, bytes: &mut I) -> Option<Ref>
        where
            I: Iterator<Item=u8>,
    {
        let val = self.read_u64(bytes)?;
        Some(Ref(val as usize))
    }

    pub fn short_or_read<T, I>(self, bytes: &mut I) -> Option<T>
        where
            I: Iterator<Item=u8>,
            T: From<u8>,
            Self: Read<T>,
    {
        if self.spec.is_long() {
            Some(self.read(bytes)?)
        } else {
            Some(self.spec.0.into())
        }
    }
}

pub trait Read<T> {
    fn read<I>(self, bytes: &mut I) -> Option<T>
        where
            I: Iterator<Item=u8>;
}

impl Read<u64> for SpecField {
    fn read<I>(self, bytes: &mut I) -> Option<u64>
        where
            I: Iterator<Item=u8>,
    { self.read_u64(bytes) }
}

impl Read<Value> for SpecField {
    fn read<I>(self, bytes: &mut I) -> Option<Value>
        where
            I: Iterator<Item=u8>,
    { self.read_value(bytes) }
}

impl Read<RefRet> for SpecField {
    fn read<I>(self, bytes: &mut I) -> Option<RefRet>
        where
            I: Iterator<Item=u8>,
    { self.read_retval(bytes) }
}

impl Read<Ref> for SpecField {
    fn read<I>(self, bytes: &mut I) -> Option<Ref>
        where
            I: Iterator<Item=u8>,
    { self.read_ref(bytes) }
}
