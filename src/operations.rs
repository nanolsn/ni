mod op_codes {
    pub const NOP: u8 = 0x00;
    pub const STOP: u8 = 0x01;
    pub const WAIT: u8 = 0x02;
    pub const SET: u8 = 0x03;
    pub const ADD: u8 = 0x04;
    pub const SUB: u8 = 0x05;
    pub const MUL: u8 = 0x06;
    pub const DIV: u8 = 0x07;
    pub const MOD: u8 = 0x08;
    pub const SHL: u8 = 0x09;
    pub const SHR: u8 = 0x0A;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operand {
    Loc(usize),
    Ind(usize),
    Ret(usize),
    Val(usize),
    Ref(usize),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BinOp {
    first: Operand,
    first_offset: Option<Operand>,
    second: Operand,
    second_offset: Option<Operand>,
}

impl BinOp {
    pub fn bin(first: Operand, second: Operand) -> Self {
        Self {
            first,
            first_offset: None,
            second,
            second_offset: None,
        }
    }

    pub fn with_first_offset(mut self, first_offset: Operand) -> Self {
        self.first_offset = Some(first_offset);
        self
    }

    pub fn with_second_offset(mut self, second_offset: Operand) -> Self {
        self.second_offset = Some(second_offset);
        self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Nop,
    Stop,
    Wait,
    Set(BinOp),
    Add(BinOp),
    Sub(BinOp),
    Mul(BinOp),
    Div(BinOp),
    Mod(BinOp),
    Shl(BinOp),
    Shr(BinOp),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

impl std::convert::TryFrom<u8> for OpType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use OpType::*;

        return Ok(match value {
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
            _ => return Err(()),
        });
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Overflowed,
    Saturated,
    Wide,
    Trigger,
}

impl Default for Mode {
    fn default() -> Self { Mode::Overflowed }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Operation {
    op: Op,
    op_type: OpType,
    mode: Mode,
}
