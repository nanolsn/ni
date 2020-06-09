use std::io::{self, Write};
use common::*;
use super::encode::*;

#[derive(Debug)]
pub enum EncodeError {
    WriteError(io::Error),
    FailedToWrite,
}

impl From<io::Error> for EncodeError {
    fn from(e: io::Error) -> Self { EncodeError::WriteError(e) }
}

pub fn encode_op<W>(op: Op, buf: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
{
    op.op_code().encode(buf)
}

impl Encode for u8 {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        match buf.write(&[*self]) {
            Ok(1) => Ok(()),
            Ok(_) => Err(EncodeError::FailedToWrite),
            Err(e) => Err(e.into()),
        }
    }
}
