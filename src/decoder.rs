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

pub fn decode_binop<F>(f: F, input: &mut ByteIterator) -> Result<Op, OpDecodeError>
    where
        F: FnOnce(Ref, Value, Option<Ref>, OpSize) -> Op,
{
    let spec = Spec(input.next().ok_or(OpExpected::Spec(0))?);

    let z = if spec.is_next() {
        let spec = Spec(input.next().ok_or(OpExpected::Spec(1))?);

        let z = spec.x().short_or_read(input).ok_or(OpExpected::Operand(2))?;
        Some(z)
    } else {
        None
    };

    let y = spec.y().read(input).ok_or(OpExpected::Operand(1))?;
    let x = spec.x().read(input).ok_or(OpExpected::Operand(0))?;
    let op_size = spec.z().to_op_size();

    Ok(f(x, y, z, op_size))
}

pub fn decode_op(input: &mut ByteIterator) -> Result<Op, OpDecodeError> {
    let op_code = input.next().ok_or(OpExpected::OpCode)?;

    match op_code {
        NOP => Ok(Op::Nop),
        STOP => {
            let spec = Spec(input.next().ok_or(OpExpected::Spec(0))?);

            let val = spec.x().short_or_read(input).ok_or(OpExpected::Operand(0))?;

            Ok(Op::Stop(val))
        }
        WAIT => {
            let spec = Spec(input.next().ok_or(OpExpected::Spec(0))?);

            let val = spec.x().short_or_read(input).ok_or(OpExpected::Operand(0))?;

            Ok(Op::Wait(val))
        }
        SET => {
            let spec = Spec(input.next().ok_or(OpExpected::Spec(0))?);

            let y = spec.y().read(input).ok_or(OpExpected::Operand(1))?;
            let x = spec.x().read(input).ok_or(OpExpected::Operand(0))?;
            let op_size = spec.z().to_op_size();

            Ok(Op::Set(x, y, op_size))
        }
        ADD => decode_binop(Op::Add, input),
        SUB => decode_binop(Op::Sub, input),
        MUL => decode_binop(Op::Mul, input),
        DIV => decode_binop(Op::Div, input),
        MOD => decode_binop(Op::Mod, input),
        MULS => decode_binop(Op::Muls, input),
        DIVS => decode_binop(Op::Divs, input),
        MODS => decode_binop(Op::Mods, input),
        SHL => decode_binop(Op::Shl, input),
        SHR => decode_binop(Op::Shr, input),
        _ => Err(UnknownOpCode),
    }
}
