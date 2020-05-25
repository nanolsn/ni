use super::*;

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    UnexpectedEnd,
    UnknownOpCode,
    UndefinedOperation(UndefinedOperation),
}

impl From<UndefinedOperation> for DecodeError {
    fn from(e: UndefinedOperation) -> Self { DecodeError::UndefinedOperation(e) }
}

pub fn decode_op<I>(bytes: &mut I) -> Result<Op, DecodeError>
    where
        I: Iterator<Item=u8>,
{
    use op_codes::*;

    let op_code = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;

    let op = match op_code {
        NOP => Op::Nop,
        STOP => {
            let x = decode_operand(bytes)?;
            Op::Stop(UnOp::new(x))
        }
        WAIT => {
            let x = decode_operand(bytes)?;
            Op::Wait(UnOp::new(x))
        }
        SET => {
            let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;
            let Spec { op_type, mode, variant } = decode_spec(byte)?;
            let bin_op = decode_bin_op(bytes, variant)?;

            Op::Set(bin_op, op_type, mode)
        }
        _ => return Err(DecodeError::UnknownOpCode),
    };

    Ok(op)
}

fn decode_bin_op<I>(bytes: &mut I, variant: Variant) -> Result<BinOp, DecodeError>
    where
        I: Iterator<Item=u8>,
{
    let x = decode_operand(bytes)?;
    let y = decode_operand(bytes)?;
    let bin_op = BinOp::bin(x, y);

    Ok(match variant {
        Variant::XY => bin_op,
        Variant::XOffsetY => {
            let x_offset = decode_operand(bytes)?;

            bin_op.with_x_offset(x_offset)
        }
        Variant::XYOffset => {
            let y_offset = decode_operand(bytes)?;

            bin_op.with_y_offset(y_offset)
        }
        Variant::XOffsetYOffset => {
            let x_offset = decode_operand(bytes)?;
            let y_offset = decode_operand(bytes)?;

            bin_op
                .with_x_offset(x_offset)
                .with_y_offset(y_offset)
        }
    })
}

fn decode_spec(byte: u8) -> Result<Spec, DecodeError> {
    const OP_TYPE_BITS: u8 = 0b0000_1111;
    const MODE_BITS: u8 = 0b0011_0000;
    const VARIANT_BITS: u8 = 0b1100_0000;

    let op_type = OpType::new(byte & OP_TYPE_BITS)?;
    let mode = Mode::new((byte & MODE_BITS) >> 4)?;
    let variant = Variant::new((byte & VARIANT_BITS) >> 6)?;

    Ok(Spec { op_type, mode, variant })
}

pub fn decode_operand<I>(bytes: &mut I) -> Result<Operand, DecodeError>
    where
        I: Iterator<Item=u8>,
{
    use std::mem::size_of;

    const SIZE_BITS: u8 = 0b0000_1111;
    const KIND_BITS: u8 = 0b0111_0000;

    let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;

    if let Some(operand) = decode_short_operand(byte) {
        return Ok(operand);
    }

    let size = (byte & SIZE_BITS) as usize;
    let mut buf: [u8; size_of::<usize>()] = [0; size_of::<usize>()];

    for i in 0..size {
        buf[i] = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;
    }

    let value = usize::from_le_bytes(buf);
    let kind = (byte & KIND_BITS) >> 4;

    Ok(Operand::new(value, kind)?)
}

fn decode_short_operand(byte: u8) -> Option<Operand> {
    const SHORT_OPERAND_BIT: u8 = 0b1000_0000;

    return if byte & SHORT_OPERAND_BIT == 0 {
        None
    } else {
        Some((byte & !SHORT_OPERAND_BIT).into())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        // TODO
        assert!(true);
    }
}
