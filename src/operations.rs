#[derive(Debug, Eq, PartialEq)]
pub enum UndefinedOperation {
    Mode,
    OpType,
    Kind,
    Variant,
}

pub mod op_codes {
    pub const NOP: u8 = 0x00;
    pub const END: u8 = 0x01;
    pub const SLP: u8 = 0x02;
    pub const SET: u8 = 0x03;
    pub const ADD: u8 = 0x04;
    pub const SUB: u8 = 0x05;
    pub const MUL: u8 = 0x06;
    pub const DIV: u8 = 0x07;
    pub const MOD: u8 = 0x08;
    pub const SHL: u8 = 0x09;
    pub const SHR: u8 = 0x0A;
    pub const AND: u8 = 0x0B;
    pub const OR: u8 = 0x0C;
    pub const XOR: u8 = 0x0D;
    pub const NOT: u8 = 0x0E;
    pub const NEG: u8 = 0x0F;
    pub const INC: u8 = 0x10;
    pub const DEC: u8 = 0x11;
}

#[derive(Debug, Eq, PartialEq)]
pub enum Operand {
    Loc(usize),
    Ind(usize),
    Ret(usize),
    Val(usize),
    Ref(usize),
}

impl Operand {
    pub fn new(value: usize, kind: u8) -> Result<Self, UndefinedOperation> {
        use Operand::*;

        Ok(match kind {
            0 => Loc(value),
            1 => Ind(value),
            2 => Ret(value),
            3 => Val(value),
            4 => Ref(value),
            _ => return Err(UndefinedOperation::Kind),
        })
    }
}

impl From<u8> for Operand {
    fn from(byte: u8) -> Self { Operand::Loc(byte as usize) }
}

#[derive(Debug, Eq, PartialEq)]
pub struct UnOp {
    x: Operand,
    x_offset: Option<Operand>,
}

impl UnOp {
    pub fn new(x: Operand) -> Self { Self { x, x_offset: None } }

    pub fn with_x_offset(mut self, x_offset: Operand) -> Self {
        self.x_offset = Some(x_offset);
        self
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BinOp {
    x: Operand,
    x_offset: Option<Operand>,
    y: Operand,
    y_offset: Option<Operand>,
}

impl BinOp {
    pub fn new(x: Operand, y: Operand) -> Self {
        Self {
            x,
            x_offset: None,
            y,
            y_offset: None,
        }
    }

    pub fn with_x_offset(mut self, x_offset: Operand) -> Self {
        self.x_offset = Some(x_offset);
        self
    }

    pub fn with_y_offset(mut self, y_offset: Operand) -> Self {
        self.y_offset = Some(y_offset);
        self
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Op {
    Nop,
    End(UnOp),
    Slp(UnOp),
    Set(BinOp, OpType),
    Add(BinOp, OpType, Mode),
    Sub(BinOp, OpType, Mode),
    Mul(BinOp, OpType, Mode),
    Div(BinOp, OpType),
    Mod(BinOp, OpType),
    Shl(BinOp, OpType, Mode),
    Shr(BinOp, OpType, Mode),
    And(BinOp, OpType),
    Or(BinOp, OpType),
    Xor(BinOp, OpType),
    Not(UnOp, OpType),
    Neg(UnOp, OpType, Mode),
    Inc(UnOp, OpType, Mode),
    Dec(UnOp, OpType, Mode),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Spec {
    pub op_type: OpType,
    pub mode: Mode,
    pub variant: Variant,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OpType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Uw,
    Iw,
}

impl OpType {
    pub fn new(value: u8) -> Result<Self, UndefinedOperation> {
        use OpType::*;

        Ok(match value {
            0 => U8,
            1 => I8,
            2 => U16,
            3 => I16,
            4 => U32,
            5 => I32,
            6 => U64,
            7 => I64,
            8 => Uw,
            9 => Iw,
            _ => return Err(UndefinedOperation::OpType),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Mode {
    Wrap,
    Sat,
    Wide,
    Hand,
}

impl Mode {
    pub fn new(value: u8) -> Result<Self, UndefinedOperation> {
        use Mode::*;

        Ok(match value {
            0 => Wrap,
            1 => Sat,
            2 => Wide,
            3 => Hand,
            _ => return Err(UndefinedOperation::Mode),
        })
    }
}

impl Default for Mode {
    fn default() -> Self { Mode::Wrap }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    XY,
    XOffsetY,
    XYOffset,
    XOffsetYOffset,
}

impl Variant {
    pub fn new(variant: u8) -> Result<Self, UndefinedOperation> {
        use Variant::*;

        Ok(match variant {
            0 => XY,
            1 => XOffsetY,
            2 => XYOffset,
            3 => XOffsetYOffset,
            _ => return Err(UndefinedOperation::Variant),
        })
    }
}
