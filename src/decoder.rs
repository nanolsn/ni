use super::operation::*;
use super::instruction::*;
use super::spec::*;
use super::byte_iterator::ByteIterator;

#[derive(Debug, Eq, PartialEq)]
pub enum OpExpected {
    Instruction,
    Spec(usize),
    Operand(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub enum OpDecodeError {
    UnknownOpCode,
    UnexpectedInputEnd,
}

impl From<OpExpected> for OpDecodeError {
    fn from(_: OpExpected) -> Self { UnexpectedInputEnd }
}

use OpDecodeError::*;

#[derive(Debug)]
pub struct Decoder<'c> {
    code: ByteIterator<'c>,
}

impl<'c> Decoder<'c> {
    pub fn new(code: &'c [u8]) -> Self { Decoder { code: ByteIterator::new(code) } }

    pub fn decode(&mut self) -> Result<Op, OpDecodeError> { decode_op(&mut self.code) }

    pub fn end(&self) -> bool { self.code.end() }
}

impl<'c> Iterator for Decoder<'c> {
    type Item = Result<Op, OpDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.end() {
            Some(self.decode())
        } else {
            None
        }
    }
}

pub fn decode_bin_op<I>(input: &mut I) -> Option<(Ref, Value, Option<Ref>, OpSize)>
    where
        I: Iterator<Item=u8>,
{
    let spec = Spec(input.next()?);

    let z = spec.and_next(input, |spec, input| {
        Some(spec.x().short_or_read(input)?)
    });

    let y = spec.y().read(input)?;
    let x = spec.x().read(input)?;
    let op_size = spec.z().to_op_size();

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
            let op_size = spec.z().to_op_size();

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
