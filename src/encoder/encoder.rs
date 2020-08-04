use std::io::{self, Write};
use crate::common::{*, bits::LONG_OPERAND_BIT};
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

fn encode_op<W>(op: Op, buf: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
{
    use op_codes::*;
    use Op::*;

    match op {
        Nop => NOP.encode(buf),
        End(x) => {
            END.encode(buf)?;
            x.encode(buf)
        }
        Slp(x) => {
            SLP.encode(buf)?;
            x.encode(buf)
        }
        Set(b, t) => {
            SET.encode(buf)?;
            (b, t).encode(buf)
        }
        Cnv(x, y, t, u) => {
            CNV.encode(buf)?;
            (t, u).encode(buf)?;
            x.encode(buf)?;
            y.encode(buf)
        }
        Add(b, t) => {
            ADD.encode(buf)?;
            (b, t).encode(buf)
        }
        Sub(b, t) => {
            SUB.encode(buf)?;
            (b, t).encode(buf)
        }
        Mul(b, t) => {
            MUL.encode(buf)?;
            (b, t).encode(buf)
        }
        Div(b, t) => {
            DIV.encode(buf)?;
            (b, t).encode(buf)
        }
        Mod(b, t) => {
            MOD.encode(buf)?;
            (b, t).encode(buf)
        }
        Shl(x, y, t) => {
            SHL.encode(buf)?;
            t.encode(buf)?;
            x.encode(buf)?;
            y.encode(buf)
        }
        Shr(x, y, t) => {
            SHR.encode(buf)?;
            t.encode(buf)?;
            x.encode(buf)?;
            y.encode(buf)
        }
        And(b, t) => {
            AND.encode(buf)?;
            (b, t).encode(buf)
        }
        Or(b, t) => {
            OR.encode(buf)?;
            (b, t).encode(buf)
        }
        Xor(b, t) => {
            XOR.encode(buf)?;
            (b, t).encode(buf)
        }
        Not(u, t) => {
            NOT.encode(buf)?;
            (u, t).encode(buf)
        }
        Neg(u, t) => {
            NEG.encode(buf)?;
            (u, t).encode(buf)
        }
        Inc(u, t) => {
            INC.encode(buf)?;
            (u, t).encode(buf)
        }
        Dec(u, t) => {
            DEC.encode(buf)?;
            (u, t).encode(buf)
        }
        Go(x) => {
            GO.encode(buf)?;
            x.encode(buf)
        }
        Ift(u, t) => {
            IFT.encode(buf)?;
            (u, t).encode(buf)
        }
        Iff(u, t) => {
            IFF.encode(buf)?;
            (u, t).encode(buf)
        }
        Ife(b, t) => {
            IFE.encode(buf)?;
            (b, t).encode(buf)
        }
        Ifl(b, t) => {
            IFL.encode(buf)?;
            (b, t).encode(buf)
        }
        Ifg(b, t) => {
            IFG.encode(buf)?;
            (b, t).encode(buf)
        }
        Ine(b, t) => {
            INE.encode(buf)?;
            (b, t).encode(buf)
        }
        Inl(b, t) => {
            INL.encode(buf)?;
            (b, t).encode(buf)
        }
        Ing(b, t) => {
            ING.encode(buf)?;
            (b, t).encode(buf)
        }
        Ifa(b, t) => {
            IFA.encode(buf)?;
            (b, t).encode(buf)
        }
        Ifo(b, t) => {
            IFO.encode(buf)?;
            (b, t).encode(buf)
        }
        Ifx(b, t) => {
            IFX.encode(buf)?;
            (b, t).encode(buf)
        }
        Ina(b, t) => {
            INA.encode(buf)?;
            (b, t).encode(buf)
        }
        Ino(b, t) => {
            INO.encode(buf)?;
            (b, t).encode(buf)
        }
        Inx(b, t) => {
            INX.encode(buf)?;
            (b, t).encode(buf)
        }
        App(x) => {
            APP.encode(buf)?;
            x.encode(buf)
        }
        Par(u, t, m) => {
            PAR.encode(buf)?;
            (u, t, m.as_mode()).encode(buf)
        }
        Clf(x) => {
            CLF.encode(buf)?;
            x.encode(buf)
        }
        Ret(u, t) => {
            RET.encode(buf)?;
            (u, t).encode(buf)
        }
        In(b) => {
            IN.encode(buf)?;
            (b, OpType::U8).encode(buf)
        }
        Out(u) => {
            OUT.encode(buf)?;
            (u, OpType::U8).encode(buf)
        }
        Fls => FLS.encode(buf),
        Sfd(x) => {
            SFD.encode(buf)?;
            x.encode(buf)
        }
        Gfd(x) => {
            GFD.encode(buf)?;
            x.encode(buf)
        }
        Zer(x, y) => {
            ZER.encode(buf)?;
            x.encode(buf)?;
            y.encode(buf)
        }
        Cmp(x, y, z) => {
            CMP.encode(buf)?;
            x.encode(buf)?;
            y.encode(buf)?;
            z.encode(buf)
        }
        Cpy(x, y, z) => {
            CPY.encode(buf)?;
            x.encode(buf)?;
            y.encode(buf)?;
            z.encode(buf)
        }
    }
}

impl Encode for Op {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    { encode_op(*self, buf) }
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
        const SHORT_MAX_VALUE: UWord = !LONG_OPERAND_BIT as UWord;

        if let Some(val) = self.get() {
            let bytes = val.to_le_bytes();

            if val <= SHORT_MAX_VALUE && matches!(self, Operand::Loc(_)) {
                return bytes[0].encode(buf);
            }

            let n_bytes = bytes
                .iter()
                .rev()
                .skip_while(|&b| *b == 0)
                .count();

            let mut meta = self.as_byte() << 4;
            meta |= n_bytes as u8 - 1;
            meta |= LONG_OPERAND_BIT;

            meta.encode(buf)?;
            buf.write(&bytes[..n_bytes]).expected(n_bytes)
        } else {
            let operand_meta = self.as_byte() << 4;
            operand_meta.encode(buf)
        }
    }
}

impl Encode for OpType {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    { self.as_byte().encode(buf) }
}

impl Encode for (OpType, Mode, Variant) {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (op_type, Mode(mode), variant) = self;
        let mut meta = variant.as_byte() << 6;
        meta |= *mode << 4;
        meta |= op_type.as_byte();

        meta.encode(buf)
    }
}

impl Encode for (UnOp, OpType, Mode) {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (un_op, op_type, mode) = self;

        (*op_type, *mode, un_op.variant()).encode(buf)?;
        un_op.encode(buf)
    }
}

impl Encode for (BinOp, OpType) {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (bin_op, op_type) = self;

        (*op_type, Mode(0), bin_op.variant()).encode(buf)?;
        bin_op.encode(buf)
    }
}

impl Encode for (UnOp, OpType) {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (un_op, op_type) = self;
        (*un_op, *op_type, Mode(0)).encode(buf)
    }
}

impl Encode for (OpType, OpType) {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (t, u) = self;
        let mut meta = t.as_byte();
        meta |= u.as_byte() << 4;

        meta.encode(buf)
    }
}

impl Encode for BinOp {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (x, y, offset) = match self {
            BinOp::None { x, y } => (x, y, None),
            BinOp::First { x, y, offset } => (x, y, Some(offset)),
            BinOp::Second { x, y, offset } => (x, y, Some(offset)),
            BinOp::Both { x, y, offset } => (x, y, Some(offset)),
        };

        x.encode(buf)?;
        y.encode(buf)?;

        if let Some(o) = offset {
            o.encode(buf)
        } else {
            Ok(())
        }
    }
}

impl Encode for UnOp {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (x, offset) = match self {
            UnOp::None { x } => (x, None),
            UnOp::First { x, offset } => (x, Some(offset)),
        };

        x.encode(buf)?;

        if let Some(o) = offset {
            o.encode(buf)
        } else {
            Ok(())
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
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[END, 12]);
    }

    #[test]
    fn encode_long() {
        let op = Op::End(Operand::Ind(12));

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[END, 0b1001_0000, 12]);

        let op = Op::End(Operand::Val(256));

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[END, 0b1011_0001, 0, 1]);
    }

    #[test]
    fn encode_emp() {
        let op = Op::End(Operand::Emp);

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[END, 0b0110_0000]);
    }

    #[test]
    fn encode_un_first_offset() {
        let op = Op::Inc(
            UnOp::new(Operand::Ind(16)).with_first(Operand::Ref(1)),
            OpType::I16,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[INC, 0b0100_0011, 0b1001_0000, 16, 0b1100_0000, 1]);
    }

    #[test]
    fn encode_bin_short() {
        let op = Op::Set(
            BinOp::new(Operand::Loc(8), Operand::Loc(16)),
            OpType::I16,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[SET, 0b0000_0011, 8, 16]);
    }

    #[test]
    fn encode_bin_long() {
        let op = Op::Add(
            BinOp::new(Operand::Loc(256), Operand::Ind(257)),
            OpType::U32,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[ADD, 0b0000_0100, 0b1000_0001, 0, 1, 0b1001_0001, 1, 1]);
    }

    #[test]
    fn encode_bin_first_offset() {
        let op = Op::Set(
            BinOp::new(Operand::Ret(8), Operand::Ref(16)).with_first(Operand::Val(5)),
            OpType::U32,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[SET, 0b0100_0100, 0b1010_0000, 8, 0b1100_0000, 16, 0b1011_0000, 5]);
    }

    #[test]
    fn encode_bin_second_offset() {
        let op = Op::Div(
            BinOp::new(Operand::Ret(8), Operand::Ref(16)).with_second(Operand::Val(5)),
            OpType::U32,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[DIV, 0b1000_0100, 0b1010_0000, 8, 0b1100_0000, 16, 0b1011_0000, 5]);
    }

    #[test]
    fn encode_bin_both_offset() {
        let op = Op::Mod(
            BinOp::new(Operand::Ret(8), Operand::Ref(16)).with_both(Operand::Val(5)),
            OpType::U32,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[MOD, 0b1100_0100, 0b1010_0000, 8, 0b1100_0000, 16, 0b1011_0000, 5]);
    }

    #[test]
    fn encode_cnv() {
        let op = Op::Cnv(Operand::Loc(12), Operand::Loc(9), OpType::U8, OpType::U16);

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[CNV, 0b0010_0000, 12, 9]);
    }

    #[test]
    fn encode_shl() {
        let op = Op::Shl(Operand::Loc(12), Operand::Loc(9), OpType::U32);

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[SHL, 0b0000_0100, 12, 9]);
    }

    #[test]
    fn encode_ife() {
        let op = Op::Ife(
            BinOp::new(Operand::Loc(12), Operand::Ref(8)).with_first(Operand::Ref(4)),
            OpType::U16,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[IFE, 0b0100_0010, 12, 0b1100_0000, 8, 0b1100_0000, 4]);
    }

    #[test]
    fn encode_app() {
        let op = Op::App(Operand::Ref(8));

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[APP, 0b1100_0000, 8]);
    }

    #[test]
    fn encode_par() {
        let op = Op::Par(
            UnOp::new(Operand::Ref(8)).with_first(Operand::Val(6)),
            OpType::F32,
            ParameterMode::Emp,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[PAR, 0b0101_1011, 0b1100_0000, 8, 0b1011_0000, 6]);
    }

    #[test]
    fn encode_ret() {
        let op = Op::Ret(UnOp::new(Operand::Loc(16)), OpType::U8);

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[RET, 0b0000_0000, 16]);
    }

    #[test]
    fn encode_in() {
        let op = Op::In(BinOp::new(Operand::Loc(0), Operand::Loc(2))
            .with_both(Operand::Loc(1))
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[IN, 0b1100_0000, 0, 2, 1]);
    }

    #[test]
    fn encode_out() {
        let op = Op::Out(UnOp::new(Operand::Loc(0)).with_first(Operand::Loc(1)));

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[OUT, 0b0100_0000, 0, 1]);
    }

    #[test]
    fn encode_fls() {
        let op = Op::Fls;

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[FLS]);
    }

    #[test]
    fn encode_cpy() {
        let op = Op::Cpy(Operand::Loc(0), Operand::Loc(1), Operand::Val(12));

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[CPY, 0, 1, 0b1011_0000, 12]);
    }
}
