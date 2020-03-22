const SIZE_MASK: u8 = 0b0000_0011;

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

use super::operation::*;
use super::instruction::*;
use super::spec::Spec;

#[derive(Debug, Eq, PartialEq)]
pub enum OpDecodeError {
    UnknownOpCode,
    UnexpectedInputEnd,
    IncorrectOpFormat(u8),
}

use OpDecodeError::*;

pub fn decode_bin_op<I>(input: &mut I) -> Option<(Ref, Value, Option<Ref>, usize)>
    where
        I: Iterator<Item=u8>,
{
    let spec = Spec(input.next()?);

    let z = if spec.is_next() {
        let spec = Spec(input.next()?);

        let z = spec.x().short_or_read(input)?;
        Some(z)
    } else {
        None
    };

    let y = spec.y().read(input)?;
    let x = spec.x().read(input)?;
    let op_size = spec.z().to_size();

    Some((x, y, z, op_size))
}

pub fn decode_op<I>(input: &mut I) -> Result<Op, OpDecodeError>
    where
        I: Iterator<Item=u8>,
{
    let op_code = input.next().ok_or(UnexpectedInputEnd)?;

    match op_code {
        NOP => Ok(Op::Nop),
        STOP => {
            let spec = Spec(input.next().ok_or(UnexpectedInputEnd)?);

            let val = spec.x().short_or_read(input).ok_or(UnexpectedInputEnd)?;

            Ok(Op::Stop(val))
        }
        WAIT => {
            let spec = Spec(input.next().ok_or(UnexpectedInputEnd)?);

            let val = spec.x().short_or_read(input).ok_or(UnexpectedInputEnd)?;

            Ok(Op::Wait(val))
        }
        SET => {
            let spec = Spec(input.next().ok_or(UnexpectedInputEnd)?);

            let y = spec.y().read(input).ok_or(UnexpectedInputEnd)?;
            let x = spec.x().read(input).ok_or(UnexpectedInputEnd)?;
            let op_size = spec.z().to_size();

            Ok(Op::Set(x, y, op_size))
        }
        ADD => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Add(x, y, z, op_size))
        }
        SUB => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Sub(x, y, z, op_size))
        }
        MUL => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Mul(x, y, z, op_size))
        }
        DIV => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Div(x, y, z, op_size))
        }
        MOD => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Mod(x, y, z, op_size))
        }
        MULS => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Muls(x, y, z, op_size))
        }
        DIVS => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Divs(x, y, z, op_size))
        }
        MODS => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Mods(x, y, z, op_size))
        }
        SHL => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Shl(x, y, z, op_size))
        }
        SHR => {
            let (x, y, z, op_size) = decode_bin_op(input)
                .ok_or(UnexpectedInputEnd)?;

            Ok(Op::Shr(x, y, z, op_size))
        }
        _ => Err(UnknownOpCode),
    }
}
