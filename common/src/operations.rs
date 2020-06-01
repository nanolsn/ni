#[derive(Debug, Eq, PartialEq)]
pub enum UndefinedOperation {
    OpType,
    Kind,
    Variant,
    ArithmeticMode,
    ParameterMode,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Operand {
    /// Local variable.
    ///
    /// Expressed as `x` or `loc(12)`.
    Loc(usize),

    /// Indirection access.
    ///
    /// Expressed as `*x` or `ind(12)`.
    Ind(usize),

    /// Return variable.
    ///
    /// Expressed as `^x` or `ret(12)`.
    Ret(usize),

    /// Constant value.
    ///
    /// Expressed as `12` or `val(12)`.
    Val(usize),

    /// Variable reference.
    ///
    /// Expressed as `&x` or `ref(12)`.
    Ref(usize),

    /// Empty.
    ///
    /// Expressed as `emp`.
    Emp,
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
            5 => Emp,
            _ => return Err(UndefinedOperation::Kind),
        })
    }

    pub fn get(self) -> Option<usize> {
        use Operand::*;

        match self {
            Loc(v) => Some(v),
            Ind(v) => Some(v),
            Ret(v) => Some(v),
            Val(v) => Some(v),
            Ref(v) => Some(v),
            Emp => None,
        }
    }

    pub fn map<F>(self, f: F) -> Self
        where
            F: FnOnce(usize) -> usize,
    {
        use Operand::*;

        match self {
            Loc(v) => Operand::Loc(f(v)),
            Ind(v) => Operand::Ind(f(v)),
            Ret(v) => Operand::Ret(f(v)),
            Val(v) => Operand::Val(f(v)),
            Ref(v) => Operand::Ref(f(v)),
            Emp => Operand::Emp,
        }
    }
}

impl From<u8> for Operand {
    fn from(byte: u8) -> Self { Operand::Loc(byte as usize) }
}

impl std::fmt::Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operand::*;

        match self {
            Loc(v) => write!(f, "loc({:#02X?})", v),
            Ind(v) => write!(f, "ind({:#02X?})", v),
            Ret(v) => write!(f, "ret({:#02X?})", v),
            Val(v) => write!(f, "val({:#02X?})", v),
            Ref(v) => write!(f, "ref({:#02X?})", v),
            Emp => write!(f, "emp"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct UnOp {
    pub x: Operand,
    pub x_offset: Option<Operand>,
}

impl UnOp {
    pub fn new(x: Operand) -> Self { Self { x, x_offset: None } }

    pub fn with_x_offset(mut self, x_offset: Operand) -> Self {
        self.x_offset = Some(x_offset);
        self
    }
}

impl std::fmt::Debug for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.x)?;

        if let Some(offset) = &self.x_offset {
            write!(f, ":{:?}", offset)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BinOp {
    pub x: Operand,
    pub x_offset: Option<Operand>,
    pub y: Operand,
    pub y_offset: Option<Operand>,
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

impl std::fmt::Debug for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.x)?;

        if let Some(offset) = &self.x_offset {
            write!(f, ":{:?}", offset)?;
        }

        write!(f, " {:?}", self.y)?;

        if let Some(offset) = &self.y_offset {
            write!(f, ":{:?}", offset)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Op {
    Nop,
    End(Operand),
    Slp(Operand),
    Set(BinOp, OpType),
    Add(BinOp, OpType, ArithmeticMode),
    Sub(BinOp, OpType, ArithmeticMode),
    Mul(BinOp, OpType, ArithmeticMode),
    Div(BinOp, OpType),
    Mod(BinOp, OpType),
    Shl(BinOp, OpType, ArithmeticMode),
    Shr(BinOp, OpType, ArithmeticMode),
    And(BinOp, OpType),
    Or(BinOp, OpType),
    Xor(BinOp, OpType),
    Not(UnOp, OpType),
    Neg(UnOp, OpType, ArithmeticMode),
    Inc(UnOp, OpType, ArithmeticMode),
    Dec(UnOp, OpType, ArithmeticMode),
    Go(Operand),
    Ift(UnOp, OpType),
    Iff(UnOp, OpType),
    Ife(BinOp, OpType),
    Ifl(BinOp, OpType),
    Ifg(BinOp, OpType),
    Ine(BinOp, OpType),
    Inl(BinOp, OpType),
    Ing(BinOp, OpType),
    Ifa(BinOp, OpType),
    Ifo(BinOp, OpType),
    Ifx(BinOp, OpType),
    Ina(BinOp, OpType),
    Ino(BinOp, OpType),
    Inx(BinOp, OpType),
    App(Operand),
    Par(UnOp, OpType, ParameterMode),
    Clf(Operand),
    Ret,
}

impl Op {
    pub fn is_conditional(&self) -> bool {
        use Op::*;

        match self {
            Ift(_, _) | Iff(_, _)
            | Ife(_, _) | Ifl(_, _) | Ifg(_, _)
            | Ine(_, _) | Inl(_, _) | Ing(_, _)
            | Ifa(_, _) | Ifo(_, _) | Ifx(_, _)
            | Ina(_, _) | Ino(_, _) | Inx(_, _) => true,
            _ => false,
        }
    }

    pub fn op_code(&self) -> u8 {
        use Op::*;
        use super::op_codes::*;

        match self {
            Nop => NOP,
            End(_) => END,
            Slp(_) => SLP,
            Set(_, _) => SET,
            Add(_, _, _) => ADD,
            Sub(_, _, _) => SUB,
            Mul(_, _, _) => MUL,
            Div(_, _) => DIV,
            Mod(_, _) => MOD,
            Shl(_, _, _) => SHL,
            Shr(_, _, _) => SHR,
            And(_, _) => AND,
            Or(_, _) => OR,
            Xor(_, _) => XOR,
            Not(_, _) => NOT,
            Neg(_, _, _) => NEG,
            Inc(_, _, _) => INC,
            Dec(_, _, _) => DEC,
            Go(_) => GO,
            Ift(_, _) => IFT,
            Iff(_, _) => IFF,
            Ife(_, _) => IFE,
            Ifl(_, _) => IFL,
            Ifg(_, _) => IFG,
            Ine(_, _) => INE,
            Inl(_, _) => INL,
            Ing(_, _) => ING,
            Ifa(_, _) => IFA,
            Ifo(_, _) => IFO,
            Ifx(_, _) => IFX,
            Ina(_, _) => INA,
            Ino(_, _) => INO,
            Inx(_, _) => INX,
            App(_) => APP,
            Par(_, _, _) => PAR,
            Clf(_) => CLF,
            Ret => RET,
        }
    }
}

impl std::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Op::*;

        match self {
            Nop => write!(f, "nop"),
            End(x) => write!(f, "end {:?}", x),
            Slp(x) => write!(f, "slp {:?}", x),
            Set(b, t) => write!(f, "set {:?} {:?}", t, b),
            Add(b, t, m) => write!(f, "add {:?} {:?} {:?}", m, t, b),
            Sub(b, t, m) => write!(f, "sub {:?} {:?} {:?}", m, t, b),
            Mul(b, t, m) => write!(f, "mul {:?} {:?} {:?}", m, t, b),
            Div(b, t) => write!(f, "div {:?} {:?}", t, b),
            Mod(b, t) => write!(f, "mod {:?} {:?}", t, b),
            Shl(b, t, m) => write!(f, "shl {:?} {:?} {:?}", m, t, b),
            Shr(b, t, m) => write!(f, "shr {:?} {:?} {:?}", m, t, b),
            And(b, t) => write!(f, "and {:?} {:?}", t, b),
            Or(b, t) => write!(f, "or  {:?} {:?}", t, b),
            Xor(b, t) => write!(f, "xor {:?} {:?}", t, b),
            Not(u, t) => write!(f, "not {:?} {:?}", t, u),
            Neg(u, t, m) => write!(f, "neg {:?} {:?} {:?}", m, t, u),
            Inc(u, t, m) => write!(f, "inc {:?} {:?} {:?}", m, t, u),
            Dec(u, t, m) => write!(f, "dec {:?} {:?} {:?}", m, t, u),
            Go(x) => write!(f, "go  {:?}", x),
            Ift(u, t) => write!(f, "ift {:?} {:?}", t, u),
            Iff(u, t) => write!(f, "iff {:?} {:?}", t, u),
            Ife(b, t) => write!(f, "ife {:?} {:?}", t, b),
            Ifl(b, t) => write!(f, "ifl {:?} {:?}", t, b),
            Ifg(b, t) => write!(f, "ifg {:?} {:?}", t, b),
            Ine(b, t) => write!(f, "ine {:?} {:?}", t, b),
            Inl(b, t) => write!(f, "inl {:?} {:?}", t, b),
            Ing(b, t) => write!(f, "ing {:?} {:?}", t, b),
            Ifa(b, t) => write!(f, "ifa {:?} {:?}", t, b),
            Ifo(b, t) => write!(f, "ifo {:?} {:?}", t, b),
            Ifx(b, t) => write!(f, "ifx {:?} {:?}", t, b),
            Ina(b, t) => write!(f, "ina {:?} {:?}", t, b),
            Ino(b, t) => write!(f, "ino {:?} {:?}", t, b),
            Inx(b, t) => write!(f, "inx {:?} {:?}", t, b),
            App(x) => write!(f, "app {:?}", x),
            Par(u, t, m) => write!(f, "par {:?} {:?} {:?}", m, t, u),
            Clf(x) => write!(f, "clf {:?}", x),
            Ret => write!(f, "ret"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
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
    F32,
    F64,
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
            11 => F32,
            13 => F64,
            _ => return Err(UndefinedOperation::OpType),
        })
    }

    pub fn as_byte(&self) -> u8 {
        use OpType::*;

        match self {
            U8 => 0,
            I8 => 1,
            U16 => 2,
            I16 => 3,
            U32 => 4,
            I32 => 5,
            U64 => 6,
            I64 => 7,
            Uw => 8,
            Iw => 9,
            F32 => 11,
            F64 => 13,
        }
    }
}

impl std::fmt::Debug for OpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OpType::*;

        match self {
            U8 => write!(f, "u8 "),
            I8 => write!(f, "i8 "),
            U16 => write!(f, "u16"),
            I16 => write!(f, "i16"),
            U32 => write!(f, "u32"),
            I32 => write!(f, "i32"),
            U64 => write!(f, "u64"),
            I64 => write!(f, "i64"),
            Uw => write!(f, "uw "),
            Iw => write!(f, "iw "),
            F32 => write!(f, "f32"),
            F64 => write!(f, "f64"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Mode(pub u8);

impl Mode {
    pub fn into_arithmetic(self) -> Result<ArithmeticMode, UndefinedOperation> {
        ArithmeticMode::new(self.0)
    }

    pub fn into_parameter(self) -> Result<ParameterMode, UndefinedOperation> {
        ParameterMode::new(self.0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ArithmeticMode {
    /// Wrapping mode.
    Wrap,

    /// Saturating mode.
    Sat,

    /// Wide mode.
    Wide,

    /// Handling mode.
    Hand,
}

impl ArithmeticMode {
    pub fn new(value: u8) -> Result<Self, UndefinedOperation> {
        use ArithmeticMode::*;

        Ok(match value {
            0 => Wrap,
            1 => Sat,
            2 => Wide,
            3 => Hand,
            _ => return Err(UndefinedOperation::ArithmeticMode),
        })
    }

    pub fn as_byte(&self) -> u8 {
        use ArithmeticMode::*;

        match self {
            Wrap => 0,
            Sat => 1,
            Wide => 2,
            Hand => 3,
        }
    }
}

impl Default for ArithmeticMode {
    fn default() -> Self { ArithmeticMode::Wrap }
}

impl std::fmt::Debug for ArithmeticMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArithmeticMode::*;

        match self {
            Wrap => write!(f, "wrap"),
            Sat => write!(f, "sat "),
            Wide => write!(f, "wide"),
            Hand => write!(f, "hand"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ParameterMode {
    /// Set mode.
    Set,

    /// Empty mode.
    Emp,

    /// Memory set zeroes mode.
    Msz,
}

impl ParameterMode {
    pub fn new(value: u8) -> Result<Self, UndefinedOperation> {
        use ParameterMode::*;

        Ok(match value {
            0 => Set,
            1 => Emp,
            2 => Msz,
            _ => return Err(UndefinedOperation::ParameterMode),
        })
    }

    pub fn as_byte(&self) -> u8 {
        use ParameterMode::*;

        match self {
            Set => 0,
            Emp => 1,
            Msz => 2,
        }
    }
}

impl Default for ParameterMode {
    fn default() -> Self { ParameterMode::Set }
}

impl std::fmt::Debug for ParameterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParameterMode::*;

        match self {
            Set => write!(f, "set"),
            Emp => write!(f, "emp"),
            Msz => write!(f, "msz"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    /// `x y` variant.
    NoOffset,

    /// `x:q y` variant.
    First,

    /// `x y:q` variant.
    Second,

    /// `x:q y:w` variant.
    Both,
}

impl Variant {
    pub fn new(variant: u8) -> Result<Self, UndefinedOperation> {
        use Variant::*;

        Ok(match variant {
            0 => NoOffset,
            1 => First,
            2 => Second,
            3 => Both,
            _ => return Err(UndefinedOperation::Variant),
        })
    }
}
