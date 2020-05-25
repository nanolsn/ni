use super::*;

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    UnexpectedEnd,
    UnknownOpCode,
    UndefinedMode,
    UndefinedOpType,
}

pub fn decode_op<I>(mut bytes: I) -> Result<Op, DecodeError>
    where
        I: Iterator<Item=u8>,
{
    use op_codes::*;

    let op_code = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;

    let op = match op_code {
        NOP => Op::Nop,
        STOP => Op::Stop(UnOp::new(Operand::Val(0))),
        WAIT => Op::Wait(UnOp::new(Operand::Val(0))),
        SET => {
            let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;
            let (op_type, mode) = decode_spec(byte)?;

            let bin_op = BinOp::bin(Operand::Ind(0), Operand::Ind(0))
                .with_x_offset(Operand::Loc(0))
                .with_y_offset(Operand::Loc(0));

            Op::Set(bin_op, op_type, mode)
        }
        _ => return Err(DecodeError::UnknownOpCode),
    };

    Ok(op)
}

fn decode_spec(byte: u8) -> Result<(OpType, Mode), DecodeError> {
    use std::convert::TryFrom;

    let op_type = OpType::try_from(byte & 0x00FF)?;
    let mode = Mode::try_from(byte >> 4)?;

    Ok((op_type, mode))
}
