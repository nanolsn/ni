use super::*;

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    UnexpectedEnd,
    UnknownOpCode,
    UndefinedOperation(UndefinedOperation),
    IncorrectVariant,
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
        END => {
            let (_, un_op) = decode_spec_un_op(bytes)?;
            Op::End(un_op)
        }
        SLP => {
            let (_, un_op) = decode_spec_un_op(bytes)?;
            Op::Slp(un_op)
        }
        SET => {
            let (Spec { op_type, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Set(b, op_type)
        }
        ADD => {
            let (Spec { op_type, mode, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Add(b, op_type, mode)
        }
        SUB => {
            let (Spec { op_type, mode, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Sub(b, op_type, mode)
        }
        MUL => {
            let (Spec { op_type, mode, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Mul(b, op_type, mode)
        }
        DIV => {
            let (Spec { op_type, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Div(b, op_type)
        }
        MOD => {
            let (Spec { op_type, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Mod(b, op_type)
        }
        SHL => {
            let (Spec { op_type, mode, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Shl(b, op_type, mode)
        }
        SHR => {
            let (Spec { op_type, mode, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Shr(b, op_type, mode)
        }
        AND => {
            let (Spec { op_type, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::And(b, op_type)
        }
        OR => {
            let (Spec { op_type, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Or(b, op_type)
        }
        XOR => {
            let (Spec { op_type, .. }, b) = decode_spec_bin_op(bytes)?;
            Op::Xor(b, op_type)
        }
        NOT => {
            let (Spec { op_type, .. }, u) = decode_spec_un_op(bytes)?;
            Op::Not(u, op_type)
        }
        NEG => {
            let (Spec { op_type, mode, .. }, u) = decode_spec_un_op(bytes)?;
            Op::Neg(u, op_type, mode)
        }
        INC => {
            let (Spec { op_type, mode, .. }, u) = decode_spec_un_op(bytes)?;
            Op::Inc(u, op_type, mode)
        }
        DEC => {
            let (Spec { op_type, mode, .. }, u) = decode_spec_un_op(bytes)?;
            Op::Dec(u, op_type, mode)
        }
        _ => return Err(DecodeError::UnknownOpCode),
    };

    Ok(op)
}

fn decode_spec_bin_op<I>(bytes: &mut I) -> Result<(Spec, BinOp), DecodeError>
    where
        I: Iterator<Item=u8>,
{
    let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;
    let spec = decode_spec(byte)?;
    let bin_op = decode_bin_op(bytes, spec.variant)?;

    Ok((spec, bin_op))
}

fn decode_bin_op<I>(bytes: &mut I, variant: Variant) -> Result<BinOp, DecodeError>
    where
        I: Iterator<Item=u8>,
{
    let bin_op = BinOp::new(decode_operand(bytes)?, decode_operand(bytes)?);

    Ok(match variant {
        Variant::XY => bin_op,
        Variant::XOffsetY => bin_op.with_x_offset(decode_operand(bytes)?),
        Variant::XYOffset => bin_op.with_y_offset(decode_operand(bytes)?),
        Variant::XOffsetYOffset => bin_op
            .with_x_offset(decode_operand(bytes)?)
            .with_y_offset(decode_operand(bytes)?)
    })
}

fn decode_spec_un_op<I>(bytes: &mut I) -> Result<(Spec, UnOp), DecodeError>
    where
        I: Iterator<Item=u8>,
{
    let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;
    let spec = decode_spec(byte)?;
    let un_op = decode_un_op(bytes, spec.variant)?;

    Ok((spec, un_op))
}

fn decode_un_op<I>(bytes: &mut I, variant: Variant) -> Result<UnOp, DecodeError>
    where
        I: Iterator<Item=u8>,
{
    let un_op = UnOp::new(decode_operand(bytes)?);

    Ok(match variant {
        Variant::XY => un_op,
        Variant::XOffsetY => un_op.with_x_offset(decode_operand(bytes)?),
        _ => return Err(DecodeError::IncorrectVariant),
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

fn decode_operand<I>(bytes: &mut I) -> Result<Operand, DecodeError>
    where
        I: Iterator<Item=u8>,
{
    const SIZE_BITS: u8 = 0b0000_1111;
    const KIND_BITS: u8 = 0b0111_0000;
    const SIZEOF_USIZE: usize = std::mem::size_of::<usize>();

    let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;

    if let Some(operand) = decode_short_operand(byte) {
        return Ok(operand);
    }

    let size = (byte & SIZE_BITS) as usize + 1;
    let mut buf: [u8; SIZEOF_USIZE] = [0; SIZEOF_USIZE];

    for i in 0..size {
        buf[i] = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;
    }

    let value = usize::from_le_bytes(buf);
    let kind = (byte & KIND_BITS) >> 4;

    Ok(Operand::new(value, kind)?)
}

fn decode_short_operand(byte: u8) -> Option<Operand> {
    const LONG_OPERAND_BIT: u8 = 0b1000_0000;

    return if byte & LONG_OPERAND_BIT != 0 {
        None
    } else {
        Some((byte & !LONG_OPERAND_BIT).into())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_un_short() {
        let code = [
            0x10_u8, 0b0011_0011, 16,
            // inc hand i16 loc(16)
        ];

        let expected = Op::Inc(
            UnOp::new(Operand::Loc(16)),
            OpType::I16,
            Mode::Hand,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_un_long() {
        let code = [
            0x10_u8, 0b0011_0011, 0b1001_0000, 16,
            // inc hand i16 ind(16)
        ];

        let expected = Op::Inc(
            UnOp::new(Operand::Ind(16)),
            OpType::I16,
            Mode::Hand,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_un_xo() {
        let code = [
            0x10_u8, 0b0100_0011, 0b1001_0000, 16, 0b1100_0000, 1,
            // inc i16 ind(16):ref(1)
        ];

        let expected = Op::Inc(
            UnOp::new(Operand::Ind(16)).with_x_offset(Operand::Ref(1)),
            OpType::I16,
            Mode::default(),
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_bin_short() {
        let code = [
            0x03_u8, 0b0000_0011, 8, 16,
            // set i16 loc(8) loc(16)
        ];

        let expected = Op::Set(
            BinOp::new(Operand::Loc(8), Operand::Loc(16)),
            OpType::I16,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_bin_long() {
        let code = [
            0x04_u8, 0b0000_0100, 0b1000_0001, 8, 0, 0b1001_0000, 16,
            // add u32 loc(8) ind(16)
        ];

        let expected = Op::Add(
            BinOp::new(Operand::Loc(8), Operand::Ind(16)),
            OpType::U32,
            Mode::default(),
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_bin_xo_y() {
        let code = [
            0x03_u8, 0b0101_0100, 0b1010_0000, 8, 0b1100_0000, 16, 0b1011_0000, 5,
            // set u32 ret(8):val(5) ref(16)
        ];

        let expected = Op::Set(
            BinOp::new(Operand::Ret(8), Operand::Ref(16)).with_x_offset(Operand::Val(5)),
            OpType::U32,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_bin_x_yo() {
        let code = [
            0x07_u8, 0b1000_0100, 0b1010_0000, 8, 0b1100_0000, 16, 0b1011_0000, 5,
            // div u32 ret(8) ref(16):val(5)
        ];

        let expected = Op::Div(
            BinOp::new(Operand::Ret(8), Operand::Ref(16)).with_y_offset(Operand::Val(5)),
            OpType::U32,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_bin_xo_yo() {
        let code = [
            0x08_u8, 0b1100_0100, 0b1010_0000, 8, 0b1100_0000, 16,
            0b1011_0000, 5,
            0b1011_0000, 6,
            // mod u32 ret(8):val(5) ref(16):val(6)
        ];

        let expected = Op::Mod(
            BinOp::new(Operand::Ret(8), Operand::Ref(16))
                .with_x_offset(Operand::Val(5))
                .with_y_offset(Operand::Val(6)),
            OpType::U32,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }
}
