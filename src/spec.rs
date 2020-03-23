use super::operation::*;
use crate::decoder::{OpDecodeError, OpExpected};

const SIZE_MASK: u8 = 0b0000_0011;
const CONST_BIT: u8 = 0b0100_0000;
const NEXT_BIT: u8 = 0b1000_0000;
const RETVAL_BIT: u8 = 0b1000_0000;
const LONG_BIT: u8 = 0b1000_0000;

#[derive(Copy, Clone, Debug)]
pub struct Spec {
    byte: u8,
    idx: usize,
}

impl Spec {
    fn field(self, field: u8, idx: usize) -> SpecField { SpecField { field, idx, spec: self } }

    fn decode_x(self) -> u8 { self.byte & SIZE_MASK }
    fn decode_y(self) -> u8 { self.byte >> 2 & SIZE_MASK }
    fn decode_z(self) -> u8 { self.byte >> 4 & SIZE_MASK }
    fn decode_w(self) -> u8 { self.byte >> 6 & SIZE_MASK }

    pub fn x(self) -> SpecField { self.field(self.decode_x(), self.idx * 2) }
    pub fn y(self) -> SpecField { self.field(self.decode_y(), self.idx * 2 + 1) }
    pub fn z(self) -> SpecField { self.field(self.decode_z(), self.idx * 2 + 2) }
    pub fn w(self) -> SpecField { self.field(self.decode_w(), self.idx * 2 + 3) }

    pub fn check_bits(self, mask: u8) -> bool { self.byte & mask != 0 }

    pub fn is_const(self) -> bool { self.check_bits(CONST_BIT) }
    pub fn is_next(self) -> bool { self.check_bits(NEXT_BIT) }
    pub fn is_retval(self) -> bool { self.check_bits(RETVAL_BIT) }
    pub fn is_long(self) -> bool { self.check_bits(LONG_BIT) }

    fn from_bytes<I>(bytes: &mut I, idx: usize) -> Result<Self, OpDecodeError>
        where
            I: Iterator<Item=u8>,
    {
        Ok(Spec {
            byte: bytes.next().ok_or(OpDecodeError::UnexpectedEnd(OpExpected::Spec(idx)))?,
            idx,
        })
    }

    pub fn from<I>(bytes: &mut I) -> Result<Self, OpDecodeError>
        where
            I: Iterator<Item=u8>,
    { Spec::from_bytes(bytes, 0) }

    pub fn next<I>(self, bytes: &mut I) -> Result<Option<Self>, OpDecodeError>
        where
            I: Iterator<Item=u8>,
    {
        if self.is_next() {
            Ok(Some(Spec::from_bytes(bytes, self.idx + 1)?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpecField {
    field: u8,
    idx: usize,
    spec: Spec,
}

impl SpecField {
    pub fn to_bits(self) -> u8 { self.field }

    pub fn to_op_size(self) -> OpSize { self.field.into() }

    pub fn read<T, I>(self, bytes: &mut I) -> Result<T, OpDecodeError>
        where
            I: Iterator<Item=u8>,
            Self: Read<T>,
    {
        Read::<T>::read(self, bytes)
            .ok_or(OpDecodeError::UnexpectedEnd(OpExpected::Operand(self.idx)))
    }

    fn read_u64<I>(self, bytes: &mut I) -> Option<u64>
        where
            I: Iterator<Item=u8>,
    { decode_u64(bytes, self.to_op_size().size()) }

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

    pub fn short_or_read<T, I>(self, bytes: &mut I) -> Result<T, OpDecodeError>
        where
            I: Iterator<Item=u8>,
            T: From<u8>,
            Self: Read<T>,
    {
        if self.spec.is_long() {
            Ok(self.read(bytes)?)
        } else {
            Ok(self.spec.byte.into())
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

fn decode_u64<I>(bytes: &mut I, count: usize) -> Option<u64>
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
