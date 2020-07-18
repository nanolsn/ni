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
        Add(b, t, m) => {
            ADD.encode(buf)?;
            (b, t, m.as_mode()).encode(buf)
        }
        Sub(b, t, m) => {
            SUB.encode(buf)?;
            (b, t, m.as_mode()).encode(buf)
        }
        Mul(b, t, m) => {
            MUL.encode(buf)?;
            (b, t, m.as_mode()).encode(buf)
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
        Neg(u, t, m) => {
            NEG.encode(buf)?;
            (u, t, m.as_mode()).encode(buf)
        }
        Inc(u, t, m) => {
            INC.encode(buf)?;
            (u, t, m.as_mode()).encode(buf)
        }
        Dec(u, t, m) => {
            DEC.encode(buf)?;
            (u, t, m.as_mode()).encode(buf)
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
        Ret => RET.encode(buf),
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
        const LONG_OPERAND_BIT: u8 = 0b1000_0000;
        const SHORT_MAX_VALUE: UWord = 0b0111_1111;

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

impl Encode for (BinOp, OpType, Mode) {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        let (bin_op, op_type, mode) = self;

        (*op_type, *mode, bin_op.variant()).encode(buf)?;
        bin_op.encode(buf)
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
        (*bin_op, *op_type, Mode(0)).encode(buf)
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
        self.x.encode(buf)?;
        self.y.encode(buf)?;

        if let Some(o) = self.x_offset {
            o.encode(buf)?
        }

        if let Some(o) = self.y_offset {
            o.encode(buf)?
        }

        Ok(())
    }
}

impl Encode for UnOp {
    type Err = EncodeError;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: Write,
    {
        self.x.encode(buf)?;

        if let Some(o) = self.x_offset {
            o.encode(buf)?
        }

        Ok(())
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
    fn encode_first_offset() {
        let op = Op::Inc(
            UnOp::new(Operand::Ind(16)).with_x_offset(Operand::Ref(1)),
            OpType::I16,
            ArithmeticMode::default(),
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
            ArithmeticMode::default(),
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[ADD, 0b0000_0100, 0b1000_0001, 0, 1, 0b1001_0001, 1, 1]);
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
            BinOp::new(Operand::Loc(12), Operand::Ref(8)).with_x_offset(Operand::Ref(4)),
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
            UnOp::new(Operand::Ref(8)).with_x_offset(Operand::Val(6)),
            OpType::F32,
            ParameterMode::Emp,
        );

        let mut buf = vec![];
        encode_op(op, &mut buf).unwrap();

        assert_eq!(buf, &[PAR, 0b0101_1011, 0b1100_0000, 8, 0b1011_0000, 6]);
    }
}
