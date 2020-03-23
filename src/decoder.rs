use super::operation::*;
use super::instruction::*;
use super::spec::*;
use super::byte_iterator::ByteIterator;

#[derive(Debug, Eq, PartialEq)]
pub enum OpExpected {
    OpCode,
    Spec(usize),
    Operand(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub enum OpDecodeError {
    UnknownOpCode,
    UnexpectedEnd(OpExpected),
}

impl From<OpExpected> for OpDecodeError {
    fn from(err: OpExpected) -> Self { UnexpectedEnd(err) }
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

macro_rules! decode_spec {
    ($i:ident, $n:expr) => { Spec($i.next().ok_or(OpExpected::Spec($n))?) };
    ($i:ident) => { decode_spec!($i, 0) };
}

pub fn decode_binop<I>(input: &mut I) -> Result<(Ref, Value, Option<Ref>, OpSize), OpExpected>
    where
        I: Iterator<Item=u8>,
{
    let spec = decode_spec!(input);

    let z = if spec.is_next() {
        let spec = decode_spec!(input, 1);

        let z = spec.x().short_or_read(input).ok_or(OpExpected::Operand(2))?;
        Some(z)
    } else {
        None
    };

    let y = spec.y().read(input).ok_or(OpExpected::Operand(1))?;
    let x = spec.x().read(input).ok_or(OpExpected::Operand(0))?;
    let op_size = spec.z().to_op_size();

    Ok((x, y, z, op_size))
}

pub fn decode_op<I>(input: &mut I) -> Result<Op, OpDecodeError>
    where
        I: Iterator<Item=u8>,
{
    let op_code = input.next().ok_or(OpExpected::OpCode)?;

    match op_code {
        NOP => Ok(Op::Nop),
        STOP => {
            let spec = decode_spec!(input);

            let val = spec.x().short_or_read(input).ok_or(OpExpected::Operand(0))?;

            Ok(Op::Stop(val))
        }
        WAIT => {
            let spec = decode_spec!(input);

            let val = spec.x().short_or_read(input).ok_or(OpExpected::Operand(0))?;

            Ok(Op::Wait(val))
        }
        SET => {
            let spec = decode_spec!(input);

            let y = spec.y().read(input).ok_or(OpExpected::Operand(1))?;
            let x = spec.x().read(input).ok_or(OpExpected::Operand(0))?;
            let op_size = spec.z().to_op_size();

            Ok(Op::Set(x, y, op_size))
        }
        ADD => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Add(x, y, z, op_size))
        }
        SUB => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Sub(x, y, z, op_size))
        }
        MUL => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Mul(x, y, z, op_size))
        }
        DIV => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Div(x, y, z, op_size))
        }
        MOD => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Mod(x, y, z, op_size))
        }
        MULS => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Muls(x, y, z, op_size))
        }
        DIVS => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Divs(x, y, z, op_size))
        }
        MODS => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Mods(x, y, z, op_size))
        }
        SHL => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Shl(x, y, z, op_size))
        }
        SHR => {
            let (x, y, z, op_size) = decode_binop(input)?;
            Ok(Op::Shr(x, y, z, op_size))
        }
        _ => Err(UnknownOpCode),
    }
}
