use std::io::{self, Read};
use common::*;
use super::decode::*;

#[derive(Debug)]
pub enum DecodeError {
    ReadError(io::Error),
    UnexpectedEnd,
    UnknownOpCode,
    UndefinedOperation(UndefinedOperation),
    IncorrectVariant,
}

impl From<UndefinedOperation> for DecodeError {
    fn from(e: UndefinedOperation) -> Self { DecodeError::UndefinedOperation(e) }
}

impl From<io::Error> for DecodeError {
    fn from(e: io::Error) -> Self { DecodeError::ReadError(e) }
}

impl ExpectedError for DecodeError {
    const ERROR: Self = DecodeError::UnexpectedEnd;
}

trait ReadU8 {
    fn read_u8(&mut self) -> Result<u8, DecodeError>;
}

impl<R> ReadU8 for R
    where
        R: Read,
{
    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let mut buf = [0];
        self.read(&mut buf).expected::<DecodeError>(1)?;
        Ok(buf[0])
    }
}

fn decode_op<R>(bytes: &mut R) -> Result<Op, DecodeError>
    where
        R: Read,
{
    use op_codes::*;
    use Op::*;

    let op = match bytes.read_u8()? {
        NOP => Nop,
        END => End(decode(bytes)?),
        SLP => Slp(decode(bytes)?),
        SET => {
            let (bin_op, op_type, _) = decode(bytes)?;
            Set(bin_op, op_type)
        }
        CNV => {
            let (t, u) = decode(bytes)?;
            Cnv(decode(bytes)?, decode(bytes)?, t, u)
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
            let op_type = decode(bytes)?;
            let x = decode(bytes)?;
            let y = decode(bytes)?;
            Shl(x, y, op_type)
        }
        SHR => {
            let op_type = decode(bytes)?;
            let x = decode(bytes)?;
            let y = decode(bytes)?;
            Shr(x, y, op_type)
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
        IFT => {
            let (op_type, _, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Ift(un_op, op_type)
        }
        IFF => {
            let (op_type, _, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Iff(un_op, op_type)
        }
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
        }
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
        CLF => Clf(decode(bytes)?),
        RET => {
            let (op_type, _, var) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Ret(un_op, op_type)
        }
        IN => {
            let (_, _, var): (OpType, Mode, Variant) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            In(un_op)
        }
        OUT => {
            let (_, _, var): (OpType, Mode, Variant) = decode(bytes)?;
            let un_op = decode_with(bytes, var)?;

            Out(un_op)
        }
        FLS => Fls,
        EOF => Eof(decode(bytes)?),
        _ => return Err(DecodeError::UnknownOpCode),
    };

    Ok(op)
}

impl Decode<()> for Op {
    type Err = DecodeError;

    fn decode<R>(bytes: &mut R, _: ()) -> Result<Self, Self::Err>
        where
            R: Read,
    { decode_op(bytes) }
}

impl Decode<()> for (BinOp, OpType, ArithmeticMode) {
    type Err = DecodeError;

    fn decode<R>(bytes: &mut R, _: ()) -> Result<Self, Self::Err>
        where
            R: Read,
    {
        let (op_type, mode, variant) = decode(bytes)?;
        let bin_op = decode_with(bytes, variant)?;

        Ok((bin_op, op_type, mode.into_arithmetic()?))
    }
}

impl Decode<Variant> for BinOp {
    type Err = DecodeError;

    fn decode<R>(bytes: &mut R, var: Variant) -> Result<Self, Self::Err>
        where
            R: Read,
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

    fn decode<R>(bytes: &mut R, var: Variant) -> Result<Self, Self::Err>
        where
            R: Read,
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

    fn decode<R>(bytes: &mut R, _: ()) -> Result<Self, Self::Err>
        where
            R: Read,
    {
        const OP_TYPE_BITS: u8 = 0b0000_1111;
        const MODE_BITS: u8 = 0b0011_0000;
        const VARIANT_BITS: u8 = 0b1100_0000;

        let meta = bytes.read_u8()?;

        let op_type = OpType::new(meta & OP_TYPE_BITS)?;
        let mode = Mode((meta & MODE_BITS) >> 4);
        let variant = Variant::new((meta & VARIANT_BITS) >> 6)?;

        Ok((op_type, mode, variant))
    }
}

impl Decode<()> for OpType {
    type Err = DecodeError;

    fn decode<R>(bytes: &mut R, _: ()) -> Result<Self, Self::Err>
        where
            R: Read,
    {
        const OP_TYPE_BITS: u8 = 0b0000_1111;

        let meta = bytes.read_u8()?;
        let t = OpType::new(meta & OP_TYPE_BITS)?;

        Ok(t)
    }
}

impl Decode<()> for (OpType, OpType) {
    type Err = DecodeError;

    fn decode<R>(bytes: &mut R, _: ()) -> Result<Self, Self::Err>
        where
            R: Read,
    {
        const OP_TYPE_0_BITS: u8 = 0b0000_1111;
        const OP_TYPE_1_BITS: u8 = 0b1111_0000;

        let meta = bytes.read_u8()?;

        let t = OpType::new(meta & OP_TYPE_0_BITS)?;
        let u = OpType::new((meta & OP_TYPE_1_BITS) >> 4)?;

        Ok((t, u))
    }
}

impl Decode<()> for UnOp {
    type Err = DecodeError;

    fn decode<R>(bytes: &mut R, _: ()) -> Result<Self, Self::Err>
        where
            R: Read,
    {
        let (_, _, var): (_, _, Variant) = decode(bytes)?;
        decode_with(bytes, var)
    }
}

impl Decode<()> for Operand {
    type Err = DecodeError;

    fn decode<R>(bytes: &mut R, _: ()) -> Result<Self, Self::Err>
        where
            R: Read,
    {
        const SIZE_BITS: u8 = 0b0000_1111;
        const KIND_BITS: u8 = 0b0111_0000;
        const LONG_OPERAND_BIT: u8 = 0b1000_0000;

        let meta = bytes.read_u8()?;

        if meta & LONG_OPERAND_BIT == 0 {
            return Ok((meta & !LONG_OPERAND_BIT).into());
        }

        let n_bytes = (meta & SIZE_BITS) as usize + 1;
        let mut buf = [0; std::mem::size_of::<UWord>()];

        bytes.read(&mut buf[..n_bytes]).expected::<DecodeError>(n_bytes)?;

        let value = UWord::from_le_bytes(buf);
        let kind = (meta & KIND_BITS) >> 4;

        Ok(Operand::new(value, kind)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use op_codes::*;

    #[test]
    fn decode_unexpected_end() {
        let code = [
            // inc
            INC,
        ];

        let mut code = code.as_ref();
        let actual = decode_op(&mut code);

        assert!(matches!(actual, Err(DecodeError::UnexpectedEnd)));
        assert!(code.is_empty());
    }

    #[test]
    fn decode_unknown_op_code() {
        let code = [
            // ? u16 loc(12) ref(8)
            0xFF_u8, 0b0100_0010, 12, 0b1100_0000, 8,
        ];

        let mut code = code.as_ref();
        let actual = decode_op(&mut code);

        assert!(matches!(actual, Err(DecodeError::UnknownOpCode)));
    }

    #[test]
    fn decode_incorrect_variant() {
        let code = [
            // inc u16 loc(12){loc(0)} ref(8)
            INC, 0b1000_0010, 12, 0b1100_0000, 8, 0,
        ];

        let mut code = code.as_ref();
        let actual = decode_op(&mut code);

        assert!(matches!(actual, Err(DecodeError::IncorrectVariant)));
    }

    #[test]
    fn decode_un_short() {
        let code = [
            // inc hand i16 loc(16)
            INC, 0b0011_0011, 16,
        ];

        let expected = Op::Inc(
            UnOp::new(Operand::Loc(16)),
            OpType::I16,
            ArithmeticMode::Hand,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_un_long() {
        let code = [
            // inc hand i16 ind(16)
            INC, 0b0011_0011, 0b1001_0000, 16,
        ];

        let expected = Op::Inc(
            UnOp::new(Operand::Ind(16)),
            OpType::I16,
            ArithmeticMode::Hand,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_un_first_offset() {
        let code = [
            // inc i16 ind(16){ref(1)}
            INC, 0b0100_0011, 0b1001_0000, 16, 0b1100_0000, 1,
        ];

        let expected = Op::Inc(
            UnOp::new(Operand::Ind(16)).with_x_offset(Operand::Ref(1)),
            OpType::I16,
            ArithmeticMode::default(),
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_bin_short() {
        let code = [
            // set i16 loc(8) loc(16)
            SET, 0b0000_0011, 8, 16,
        ];

        let expected = Op::Set(
            BinOp::new(Operand::Loc(8), Operand::Loc(16)),
            OpType::I16,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_bin_long() {
        let code = [
            // add u32 loc(8) ind(16)
            ADD, 0b0000_0100, 0b1000_0001, 8, 0, 0b1001_0000, 16,
        ];

        let expected = Op::Add(
            BinOp::new(Operand::Loc(8), Operand::Ind(16)),
            OpType::U32,
            ArithmeticMode::default(),
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_bin_first_offset() {
        let code = [
            // set u32 ret(8){val(5)} ref(16)
            SET, 0b0101_0100, 0b1010_0000, 8, 0b1100_0000, 16, 0b1011_0000, 5,
        ];

        let expected = Op::Set(
            BinOp::new(Operand::Ret(8), Operand::Ref(16)).with_x_offset(Operand::Val(5)),
            OpType::U32,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_bin_second_offset() {
        let code = [
            // div u32 ret(8) ref(16){val(5)}
            DIV, 0b1000_0100, 0b1010_0000, 8, 0b1100_0000, 16, 0b1011_0000, 5,
        ];

        let expected = Op::Div(
            BinOp::new(Operand::Ret(8), Operand::Ref(16)).with_y_offset(Operand::Val(5)),
            OpType::U32,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_bin_both_offset() {
        let code = [
            // mod u32 ret(8){val(5)} ref(16){val(6)}
            MOD, 0b1100_0100, 0b1010_0000, 8, 0b1100_0000, 16,
            0b1011_0000, 5,
            0b1011_0000, 6,
        ];

        let expected = Op::Mod(
            BinOp::new(Operand::Ret(8), Operand::Ref(16))
                .with_x_offset(Operand::Val(5))
                .with_y_offset(Operand::Val(6)),
            OpType::U32,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_cnv() {
        let code = [
            // cnv u8 u16 loc(12) loc(9)
            CNV, 0b0010_0000, 12, 9,
        ];

        let expected = Op::Cnv(Operand::Loc(12), Operand::Loc(9), OpType::U8, OpType::U16);

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_shl() {
        let code = [
            // shl u32 loc(12) loc(9)
            SHL, 0b0000_0100, 12, 9,
        ];

        let expected = Op::Shl(Operand::Loc(12), Operand::Loc(9), OpType::U32);

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_ife() {
        let code = [
            // ife u16 loc(12){ref(4)} ref(8)
            IFE, 0b0100_0010, 12, 0b1100_0000, 8, 0b1100_0011, 4, 0, 0, 0,
        ];

        let expected = Op::Ife(
            BinOp::new(Operand::Loc(12), Operand::Ref(8)).with_x_offset(Operand::Ref(4)),
            OpType::U16,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_ifa() {
        let code = [
            // ifa u32 loc(12) ref(8)
            IFA, 0b0000_0100, 12, 0b1100_0000, 8,
        ];

        let expected = Op::Ifa(BinOp::new(Operand::Loc(12), Operand::Ref(8)), OpType::U32);

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_app() {
        let code = [
            // app ref(8)
            APP, 0b1100_0000, 8,
        ];

        let expected = Op::App(Operand::Ref(8));

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_par() {
        let code = [
            // par emp ref(8){val(6)}
            PAR, 0b0101_1011, 0b1100_0000, 8, 0b1011_0000, 6,
        ];

        let expected = Op::Par(
            UnOp::new(Operand::Ref(8)).with_x_offset(Operand::Val(6)),
            OpType::F32,
            ParameterMode::Emp,
        );

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_ret() {
        let code = [
            // ret u8 loc(16)
            RET, 0b0000_0000, 16,
        ];

        let expected = Op::Ret(UnOp::new(Operand::Loc(16)), OpType::U8);

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_in() {
        let code = [
            // in loc(0){loc(1)}
            IN, 0b0100_0000, 0, 1,
        ];

        let expected = Op::In(UnOp::new(Operand::Loc(0)).with_x_offset(Operand::Loc(1)));

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }

    #[test]
    fn decode_fls() {
        let code = [
            // fls
            FLS,
        ];

        let expected = Op::Fls;

        let mut code = code.as_ref();
        let actual = decode_op(&mut code).unwrap();

        assert_eq!(actual, expected);
        assert!(code.is_empty());
    }
}
