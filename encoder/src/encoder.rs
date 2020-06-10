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

impl ExpectedError for EncodeError {
    const ERROR: Self = EncodeError::FailedToWrite;
}

fn encode_op<W>(op: &Op, buf: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
{
    use op_codes::*;
    use Op::*;

    match op {
        Nop => NOP.encode(buf)?,
        End(x) => {}
        _ => {}
    }

    Ok(())
}

impl Encode for u8 {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    { buf.write(&[*self]).expected(1) }
}

impl Encode for Operand {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        const SIZE_BITS: u8 = 0b0000_1111;
        const KIND_BITS: u8 = 0b0111_0000;
        const LONG_OPERAND_BIT: u8 = 0b1000_0000;
        const SHORT_MAX_VALUE: usize = 0b0111_1111;

        if let Some(val) = self.get() {
            if val <= SHORT_MAX_VALUE {
                // TODO: Encode to short.
            }

            let bytes: [u8; 8] = val.to_le_bytes();
            let size = bytes
                .iter()
                .rev()
                .take_while(|&b| *b == 0)
                .count();

            let kind = self.as_byte();
            let operand_meta = (kind << 4) | (size as u8 - 1);

            buf.write(&[operand_meta]).expected::<EncodeError>(1)?;
            buf.write(&bytes[..size]).expected(size)
        } else {
            let kind = self.as_byte();
            let operand_meta = kind << 4;

            buf.write(&[operand_meta]).expected(1)
        }
    }
}
