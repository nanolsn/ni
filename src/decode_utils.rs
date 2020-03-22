const SIZE_MASK: u8 = 0b0000_0011;
const LONG_VALUE: u8 = 0b1000_0000;
const CONST: u8 = 0b0100_0000;

pub fn is_long_value(byte: u8) -> bool { byte & LONG_VALUE != 0 }

pub fn is_const(byte: u8) -> bool { byte & CONST != 0 }

pub fn decode_x(byte: u8) -> u8 { byte & SIZE_MASK }

pub fn decode_y(byte: u8) -> u8 { byte >> 2 & SIZE_MASK }

pub fn decode_z(byte: u8) -> u8 { byte >> 4 & SIZE_MASK }

pub fn decode_w(byte: u8) -> u8 { byte >> 6 & SIZE_MASK }

pub fn to_size(n: u8) -> usize {
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

    decode_u64(bytes, to_size(decode_x(spec)))
}

use super::operation::*;

pub fn decode_value<I>(bytes: &mut I) -> Option<Value>
    where
        I: Iterator<Item=u8>,
{
    let spec = bytes.next()?;

    if !is_long_value(spec) {
        return Some(Value::Const(spec as u64));
    }

    let val = decode_u64(bytes, to_size(decode_x(spec)))?;

    if is_const(spec) {
        Some(Value::Const(val))
    } else {
        Some(Value::Ref(val as usize))
    }
}

use super::instruction::*;
use super::spec::*;

pub enum OpDecodeError {
    UnknownOpCode,
    UnexpectedInputEnd,
    IncorrectOpFormat(u8),
}

pub fn decode_op<I>(bytes: &mut I) -> Result<Op, OpDecodeError>
    where
        I: Iterator<Item=u8>,
{
    let op_code = bytes.next().ok_or(OpDecodeError::UnexpectedInputEnd)?;

    match op_code {
        NOP => Ok(Op::Nop),
        STOP => {
            let spec = SpecByte(bytes.next().ok_or(OpDecodeError::UnexpectedInputEnd)?);

            let x = spec.x_size();
            let x_val = decode_u64(bytes, x).ok_or(OpDecodeError::UnexpectedInputEnd)?;

            let val = if spec.check_bits(CONST_BIT) {
                Value::Const(x_val)
            } else {
                Value::Ref(x_val as usize)
            };

            Ok(Op::Stop(val))
        }
        WAIT => {
            let spec = SpecByte(bytes.next().ok_or(OpDecodeError::UnexpectedInputEnd)?);

            let x = spec.x_size();
            let x_val = decode_u64(bytes, x).ok_or(OpDecodeError::UnexpectedInputEnd)?;

            let val = if spec.check_bits(CONST_BIT) {
                Value::Const(x_val)
            } else {
                Value::Ref(x_val as usize)
            };

            Ok(Op::Wait(val))
        }
        SET => {
            let spec = SpecByte(bytes.next().ok_or(OpDecodeError::UnexpectedInputEnd)?);

            let x = spec.x_size();
            let y = spec.y_size();
            let op_size = spec.z_size();

            let x_val = decode_u64(bytes, x).ok_or(OpDecodeError::UnexpectedInputEnd)?;
            let y_val = decode_u64(bytes, y).ok_or(OpDecodeError::UnexpectedInputEnd)?;

            let ret_value = if spec.check_bits(RETVAL_BIT) {
                RetValue::Return(x_val as usize)
            } else {
                RetValue::Ref(x_val as usize)
            };

            let val = if spec.check_bits(CONST_BIT) {
                Value::Const(y_val)
            } else {
                Value::Ref(y_val as usize)
            };

            Ok(Op::Set(ret_value, val, op_size.into()))
        }
        _ => Err(OpDecodeError::UnknownOpCode),
    }
}
