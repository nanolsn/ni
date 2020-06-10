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
        End(x) => {
            END.encode(buf)?;
            x.encode(buf)?;
        }
        Slp(x) => {
            SLP.encode(buf)?;
            x.encode(buf)?;
        }
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
        const LONG_OPERAND_BIT: u8 = 0b1000_0000;
        const SHORT_MAX_VALUE: usize = 0b0111_1111;

        if let Some(val) = self.get() {
            let bytes = val.to_le_bytes();

            if val <= SHORT_MAX_VALUE && matches!(self, Operand::Loc(_)) {
                return buf.write(&bytes[..1]).expected(1);
            }

            let n_bytes = bytes
                .iter()
                .rev()
                .skip_while(|&b| *b == 0)
                .count();

            let kind = self.as_byte();
            let mut operand_meta = kind << 4;
            operand_meta |= n_bytes as u8 - 1;
            operand_meta |= LONG_OPERAND_BIT;

            buf.write(&[operand_meta]).expected::<EncodeError>(1)?;
            buf.write(&bytes[..n_bytes]).expected(n_bytes)
        } else {
            let kind = self.as_byte();
            let operand_meta = kind << 4;

            buf.write(&[operand_meta]).expected(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use op_codes::*;

    #[test]
    fn encode_short() {
        let op = Op::End(Operand::Loc(12));

        let mut buf = vec![];
        encode_op(&op, &mut buf).unwrap();

        assert_eq!(buf, &[END, 12]);
    }

    #[test]
    fn encode_long() {
        let op = Op::End(Operand::Ind(12));

        let mut buf = vec![];
        encode_op(&op, &mut buf).unwrap();

        assert_eq!(buf, &[END, 0b1001_0000, 12]);

        let op = Op::End(Operand::Val(256));

        let mut buf = vec![];
        encode_op(&op, &mut buf).unwrap();

        assert_eq!(buf, &[END, 0b1011_0001, 0, 1]);
    }
}
