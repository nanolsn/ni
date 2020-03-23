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

pub fn decode_binop<F>(f: F, code: &mut ByteIterator) -> Result<Op, OpDecodeError>
    where
        F: FnOnce(Ref, Value, Option<Ref>, OpSize) -> Op,
{
    let spec = Spec::from(code)?;

    let z = if let Some(spec) = spec.next(code)? {
        let z = spec.x().short_or_read(code)?;
        Some(z)
    } else {
        None
    };

    let y = spec.y().read(code)?;
    let x = spec.x().read(code)?;
    let op_size = spec.z().to_op_size();

    Ok(f(x, y, z, op_size))
}

pub fn decode_op(code: &mut ByteIterator) -> Result<Op, OpDecodeError> {
    let op_code = code.next().ok_or(OpExpected::OpCode)?;

    match op_code {
        NOP => Ok(Op::Nop),
        STOP => {
            let spec = Spec::from(code)?;
            Ok(Op::Stop(spec.x().short_or_read(code)?))
        }
        WAIT => {
            let spec = Spec::from(code)?;
            Ok(Op::Wait(spec.x().short_or_read(code)?))
        }
        SET => {
            let spec = Spec::from(code)?;

            let y = spec.y().read(code)?;
            let x = spec.x().read(code)?;
            let op_size = spec.z().to_op_size();

            Ok(Op::Set(x, y, op_size))
        }
        ADD => decode_binop(Op::Add, code),
        SUB => decode_binop(Op::Sub, code),
        MUL => decode_binop(Op::Mul, code),
        DIV => decode_binop(Op::Div, code),
        MOD => decode_binop(Op::Mod, code),
        MULS => decode_binop(Op::Muls, code),
        DIVS => decode_binop(Op::Divs, code),
        MODS => decode_binop(Op::Mods, code),
        SHL => decode_binop(Op::Shl, code),
        SHR => decode_binop(Op::Shr, code),
        _ => Err(UnknownOpCode),
    }
}
