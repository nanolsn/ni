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
    use Op::*;

    let op_code = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;

    let op = match op_code {
        NOP => Nop,
        END => End(decode(bytes)?),
        SLP => Slp(decode(bytes)?),
        SET => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Set(bin_op, op_type)
        }
        ADD => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Add(bin_op, op_type, mode)
        }
        SUB => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Sub(bin_op, op_type, mode)
        }
        MUL => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Mul(bin_op, op_type, mode)
        }
        DIV => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Div(bin_op, op_type)
        }
        MOD => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Mod(bin_op, op_type)
        }
        SHL => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Shl(bin_op, op_type, mode)
        }
        SHR => {
            let (bin_op, op_type, mode) = decode(bytes)?;
            Shr(bin_op, op_type, mode)
        }
        AND => {
            let (bin_op, op_type, _) = decode(bytes)?;
            And(bin_op, op_type)
        }
        OR => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Or(bin_op, op_type)
        }
        XOR => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Xor(bin_op, op_type)
        }
        NOT => {
            let (op_type, _, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Not(un_op, op_type)
        }
        NEG => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Neg(un_op, op_type, mode.into_arithmetic()?)
        }
        INC => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Inc(un_op, op_type, mode.into_arithmetic()?)
        }
        DEC => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Dec(un_op, op_type, mode.into_arithmetic()?)
        }
        GO => Go(decode(bytes)?),
        IFT => Ift(decode(bytes)?),
        IFF => Iff(decode(bytes)?),
        IFE => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ife(bin_op, op_type)
        }
        IFL => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ifl(bin_op, op_type)
        }
        IFG => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ifg(bin_op, op_type)
        }
        INE => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ine(bin_op, op_type)
        }
        INL => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Inl(bin_op, op_type)
        }
        ING => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ing(bin_op, op_type)
        }
        IFA => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ifa(bin_op, op_type)
        }
        IFO => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ifo(bin_op, op_type)
        }
        IFX => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ifx(bin_op, op_type)
        }
        INA => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ina(bin_op, op_type)
        }
        INO => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Ino(bin_op, op_type)
        },
        INX => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Inx(bin_op, op_type)
        }
        APP => App(decode(bytes)?),
        PAR => {
            let (op_type, mode, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Par(un_op, op_type, mode.into_parameter()?)
        }
        CFN => Cfn(decode(bytes)?),
        _ => return Err(DecodeError::UnknownOpCode),
    };

    Ok(op)
}

impl Decode<()> for Op {
    type Err = DecodeError;

    fn decode<I>(bytes: &mut I, _: ()) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>,
    { decode_op(bytes) }
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
            Variant::NoOffset => bin_op,
            Variant::First => bin_op.with_x_offset(decode(bytes)?),
            Variant::Second => bin_op.with_y_offset(decode(bytes)?),
            Variant::Both => bin_op
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
            Variant::NoOffset => un_op,
            Variant::First => un_op.with_x_offset(decode(bytes)?),
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

        let byte = bytes.next().ok_or(DecodeError::UnexpectedEnd)?;

        if byte & LONG_OPERAND_BIT == 0 {
            return Ok((byte & !LONG_OPERAND_BIT).into());
        }

        let size = (byte & SIZE_BITS) as usize + 1;
        let mut buf = [0; std::mem::size_of::<usize>()];

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
    fn decode_unexpected_end() {
        let code = [
            0x10_u8, // inc
        ];

        let expected = DecodeError::UnexpectedEnd;

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it);

        assert_eq!(actual, Err(expected));
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_unknown_op_code() {
        let code = [
            0xFF_u8, 0b0100_0010, 12, 0b1100_0000, 8,
            // ? u16 loc(12) ref(8)
        ];

        let expected = DecodeError::UnknownOpCode;

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it);

        assert_eq!(actual, Err(expected));
    }

    #[test]
    fn decode_incorrect_variant() {
        let code = [
            0x10_u8, 0b1000_0010, 12, 0b1100_0000, 8, 0,
            // inc u16 loc(12):loc(0) ref(8)
        ];

        let expected = DecodeError::IncorrectVariant;

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it);

        assert_eq!(actual, Err(expected));
    }

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
    fn decode_bin_first_offset() {
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
    fn decode_bin_second_offset() {
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
    fn decode_bin_both_offset() {
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
    fn decode_ife() {
        let code = [
            0x15_u8, 0b0100_0010, 12, 0b1100_0000, 8, 0b1100_0011, 4, 0, 0, 0,
            // ife u16 loc(12):ref(4) ref(8)
        ];

        let expected = Op::Ife(
            BinOp::new(Operand::Loc(12), Operand::Ref(8)).with_x_offset(Operand::Ref(4)),
            OpType::U16,
        );

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_ifa() {
        let code = [
            0x2B_u8, 0b0000_0100, 12, 0b1100_0000, 8,
            // ifa u32 loc(12) ref(8)
        ];

        let expected = Op::Ifa(BinOp::new(Operand::Loc(12), Operand::Ref(8)), OpType::U32);

        let mut it = code.iter().cloned();
        let actual = decode_op(&mut it).unwrap();

        assert_eq!(actual, expected);
        assert!(it.next().is_none());
    }

    #[test]
    fn decode_app() {
        let code = [
            0x31_u8, 0b1100_0000, 8,
            // app ref(8)
        ];

        let expected = Op::App(Operand::Ref(8));

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
