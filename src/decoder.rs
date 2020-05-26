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
        END => Op::End(decode(bytes)?),
        SLP => Op::Slp(decode(bytes)?),
        SET => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Op::Set(bin_op, op_type)
        }
        ADD => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Op::Add(bin_op, op_type, mode)
        }
        SUB => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Op::Sub(bin_op, op_type, mode)
        }
        MUL => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Op::Mul(bin_op, op_type, mode)
        }
        DIV => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Op::Div(bin_op, op_type)
        }
        MOD => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Op::Mod(bin_op, op_type)
        }
        SHL => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Op::Shl(bin_op, op_type, mode)
        }
        SHR => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Op::Shr(bin_op, op_type, mode)
        }
        AND => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Op::And(bin_op, op_type)
        }
        OR => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Op::Or(bin_op, op_type)
        }
        XOR => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Op::Xor(bin_op, op_type)
        }
        NOT => {
            let (op_type, _, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Op::Not(un_op, op_type)
        }
        NEG => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Op::Neg(un_op, op_type, mode.into_arithmetic()?)
        }
        INC => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Op::Inc(un_op, op_type, mode.into_arithmetic()?)
        }
        DEC => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Op::Dec(un_op, op_type, mode.into_arithmetic()?)
        }

        APP => Op::App(decode(bytes)?),
        PAR => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Op::Par(un_op, op_type, mode.into_parameter()?)
        }
        CFN => Op::Cfn(decode(bytes)?),
        _ => return Err(DecodeError::UnknownOpCode),
    };

    Ok(op)
}

impl Decode<()> for (BinOp, OpType, ArithmeticMode) {
    type Err = DecodeError;

    fn decode<I>(bytes: &mut I, _: ()) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>,
    {
        let (op_type, mode, variant) = decode(bytes)?;
        let bin_op = decode_with(bytes, variant)?;

        Ok((bin_op, op_type, mode.into_arithmetic()?))
    }
}

impl Decode<Variant> for BinOp {
    type Err = DecodeError;

    fn decode<I>(bytes: &mut I, var: Variant) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>,
    {
        let bin_op = BinOp::new(decode(bytes)?, decode(bytes)?);

        Ok(match var {
            Variant::XY => bin_op,
            Variant::XOffsetY => bin_op.with_x_offset(decode(bytes)?),
            Variant::XYOffset => bin_op.with_y_offset(decode(bytes)?),
            Variant::XOffsetYOffset => bin_op
                .with_x_offset(decode(bytes)?)
                .with_y_offset(decode(bytes)?)
        })
    }
}

impl Decode<Variant> for UnOp {
    type Err = DecodeError;

    fn decode<I>(bytes: &mut I, var: Variant) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>,
    {
        let un_op = UnOp::new(decode(bytes)?);

        Ok(match var {
            Variant::XY => un_op,
            Variant::XOffsetY => un_op.with_x_offset(decode(bytes)?),
            _ => return Err(DecodeError::IncorrectVariant),
        })
    }
}

impl Decode<()> for (OpType, Mode, Variant) {
    type Err = DecodeError;

    fn decode<I>(bytes: &mut I, _: ()) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>,
    {
        const OP_TYPE_BITS: u8 = 0b0000_1111;
        const MODE_BITS: u8 = 0b0011_0000;
        const VARIANT_BITS: u8 = 0b1100_0000;

        let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;
        let op_type = OpType::new(byte & OP_TYPE_BITS)?;
        let mode = Mode((byte & MODE_BITS) >> 4);
        let variant = Variant::new((byte & VARIANT_BITS) >> 6)?;

        Ok((op_type, mode, variant))
    }
}

impl Decode<()> for UnOp {
    type Err = DecodeError;

    fn decode<I>(bytes: &mut I, _: ()) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>,
    {
        let (_, _, var): (_, _, Variant) = decode(bytes)?;
        decode_with(bytes, var)
    }
}

impl Decode<()> for Operand {
    type Err = DecodeError;

    fn decode<I>(bytes: &mut I, _: ()) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>,
    {
        const SIZE_BITS: u8 = 0b0000_1111;
        const KIND_BITS: u8 = 0b0111_0000;
        const LONG_OPERAND_BIT: u8 = 0b1000_0000;
        const SIZEOF_USIZE: usize = std::mem::size_of::<usize>();

        let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;

        if byte & LONG_OPERAND_BIT == 0 {
            return Ok((byte & !LONG_OPERAND_BIT).into());
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
            ArithmeticMode::Hand,
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
            ArithmeticMode::Hand,
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
            ArithmeticMode::default(),
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
            ArithmeticMode::default(),
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

    #[test]
    fn decode_app() {
        let code = [
            0x31_u8, 0b0100_0000, 0b1100_0000, 8, 0b1011_0000, 6,
            // app ref(8):val(6)
        ];

        let expected = Op::App(UnOp::new(Operand::Ref(8)).with_x_offset(Operand::Val(6)));

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_par() {
        let code = [
            0x32_u8, 0b0101_1011, 0b1100_0000, 8, 0b1011_0000, 6,
            // par emp ref(8):val(6)
        ];

        let expected = Op::Par(
            UnOp::new(Operand::Ref(8)).with_x_offset(Operand::Val(6)),
            OpType::F32,
            ParameterMode::Emp,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }
}
