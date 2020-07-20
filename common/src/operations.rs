use super::{UWord, IWord};

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
    Loc(UWord),

    /// Indirection access.
    ///
    /// Expressed as `*x` or `ind(12)`.
    Ind(UWord),

    /// Return variable.
    ///
    /// Expressed as `^x` or `ret(12)`.
    Ret(UWord),

    /// Constant value.
    ///
    /// Expressed as `12` or `val(12)`.
    Val(UWord),

    /// Variable reference.
    ///
    /// Expressed as `&x` or `ref(12)`.
    Ref(UWord),

    /// Global variable.
    ///
    /// Expressed as `.x` or `glb(12)`.
    Glb(UWord),

    /// Empty.
    ///
    /// Expressed as `emp`.
    Emp,
}

impl Operand {
    pub fn new(val: UWord, kind: u8) -> Result<Self, UndefinedOperation> {
        use Operand::*;

        Ok(match kind {
            0 => Loc(val),
            1 => Ind(val),
            2 => Ret(val),
            3 => Val(val),
            4 => Ref(val),
            5 => Glb(val),
            6 => Emp,
            _ => return Err(UndefinedOperation::Kind),
        })
    }

    pub fn as_byte(&self) -> u8 {
        use Operand::*;

        match self {
            Loc(_) => 0,
            Ind(_) => 1,
            Ret(_) => 2,
            Val(_) => 3,
            Ref(_) => 4,
            Glb(_) => 5,
            Emp => 6,
        }
    }

    pub fn get(self) -> Option<UWord> {
        use Operand::*;

        match self {
            Loc(v) => Some(v),
            Ind(v) => Some(v),
            Ret(v) => Some(v),
            Val(v) => Some(v),
            Ref(v) => Some(v),
            Glb(v) => Some(v),
            Emp => None,
        }
    }

    pub fn map<F>(self, f: F) -> Self
        where
            F: FnOnce(UWord) -> UWord,
    {
        use Operand::*;

        match self {
            Loc(v) => Loc(f(v)),
            Ind(v) => Ind(f(v)),
            Ret(v) => Ret(f(v)),
            Val(v) => Val(f(v)),
            Ref(v) => Ref(f(v)),
            Glb(v) => Glb(f(v)),
            Emp => Emp,
        }
    }
}

impl From<u8> for Operand {
    fn from(byte: u8) -> Self { Operand::Loc(byte as UWord) }
}

impl std::fmt::Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operand::*;

        match self {
            Loc(v) => write!(f, "loc({:?})", v),
            Ind(v) => write!(f, "ind({:?})", v),
            Ret(v) => write!(f, "ret({:?})", v),
            Val(v) => write!(f, "val({:?})", v),
            Ref(v) => write!(f, "ref({:?})", v),
            Glb(v) => write!(f, "glb({:?})", v),
            Emp => write!(f, "emp"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnOp {
    None { x: Operand },
    First { x: Operand, offset: Operand },
}

impl UnOp {
    pub fn new(x: Operand) -> Self { UnOp::None { x } }

    pub fn with_first(self, offset: Operand) -> Self {
        if let UnOp::None { x } = self {
            UnOp::First { x, offset }
        } else {
            panic!("Wrong UnOp variant!")
        }
    }

    pub fn variant(&self) -> Variant {
        match self {
            UnOp::None { .. } => Variant::None,
            UnOp::First { .. } => Variant::First,
        }
    }

    pub fn x(&self) -> Operand {
        match self {
            UnOp::None { x } => *x,
            UnOp::First { x, .. } => *x,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinOp {
    None { x: Operand, y: Operand },
    First { x: Operand, y: Operand, offset: Operand },
    Second { x: Operand, y: Operand, offset: Operand },
    Both { x: Operand, y: Operand, offset: Operand },
}

impl BinOp {
    pub fn new(x: Operand, y: Operand) -> Self { BinOp::None { x, y } }

    pub fn with_first(self, offset: Operand) -> Self {
        if let BinOp::None { x, y } = self {
            BinOp::First { x, y, offset }
        } else {
            panic!("Wrong BinOp variant!")
        }
    }

    pub fn with_second(self, offset: Operand) -> Self {
        if let BinOp::None { x, y } = self {
            BinOp::Second { x, y, offset }
        } else {
            panic!("Wrong BinOp variant!")
        }
    }

    pub fn with_both(self, offset: Operand) -> Self {
        if let BinOp::None { x, y } = self {
            BinOp::Both { x, y, offset }
        } else {
            panic!("Wrong BinOp variant!")
        }
    }

    pub fn variant(&self) -> Variant {
        match self {
            BinOp::None { .. } => Variant::None,
            BinOp::First { .. } => Variant::First,
            BinOp::Second { .. } => Variant::Second,
            BinOp::Both { .. } => Variant::Both,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Op {
    Nop,
    End(Operand),
    Slp(Operand),
    Set(BinOp, OpType),
    Cnv(Operand, Operand, OpType, OpType),
    Add(BinOp, OpType, ArithmeticMode),
    Sub(BinOp, OpType, ArithmeticMode),
    Mul(BinOp, OpType, ArithmeticMode),
    Div(BinOp, OpType),
    Mod(BinOp, OpType),
    Shl(Operand, Operand, OpType),
    Shr(Operand, Operand, OpType),
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
    Ret(UnOp, OpType),
    In(BinOp),
    Out(UnOp),
    Fls,
    Sfd(Operand),
    Gfd(Operand),
    Zer(Operand, Operand),
    Cmp(Operand, Operand, Operand),
    Cpy(Operand, Operand, Operand),
}

impl Op {
    pub fn is_conditional(&self) -> bool {
        use Op::*;

        match self {
            Ift(..) | Iff(..)
            | Ife(..) | Ifl(..) | Ifg(..)
            | Ine(..) | Inl(..) | Ing(..)
            | Ifa(..) | Ifo(..) | Ifx(..)
            | Ina(..) | Ino(..) | Inx(..)
            | Cmp(..) => true,
            _ => false,
        }
    }

    pub fn op_code(&self) -> u8 {
        use Op::*;
        use super::op_codes::*;

        match self {
            Nop => NOP,
            End(..) => END,
            Slp(..) => SLP,
            Set(..) => SET,
            Cnv(..) => CNV,
            Add(..) => ADD,
            Sub(..) => SUB,
            Mul(..) => MUL,
            Div(..) => DIV,
            Mod(..) => MOD,
            Shl(..) => SHL,
            Shr(..) => SHR,
            And(..) => AND,
            Or(..) => OR,
            Xor(..) => XOR,
            Not(..) => NOT,
            Neg(..) => NEG,
            Inc(..) => INC,
            Dec(..) => DEC,
            Go(..) => GO,
            Ift(..) => IFT,
            Iff(..) => IFF,
            Ife(..) => IFE,
            Ifl(..) => IFL,
            Ifg(..) => IFG,
            Ine(..) => INE,
            Inl(..) => INL,
            Ing(..) => ING,
            Ifa(..) => IFA,
            Ifo(..) => IFO,
            Ifx(..) => IFX,
            Ina(..) => INA,
            Ino(..) => INO,
            Inx(..) => INX,
            App(..) => APP,
            Par(..) => PAR,
            Clf(..) => CLF,
            Ret(..) => RET,
            In(..) => IN,
            Out(..) => OUT,
            Fls => FLS,
            Sfd(..) => SFD,
            Gfd(..) => GFD,
            Zer(..) => ZER,
            Cmp(..) => CMP,
            Cpy(..) => CPY,
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
            Cnv(x, y, t, v) => write!(f, "cnv {:?} {:?} {:?} {:?}", t, v, x, y),
            Add(b, t, m) => write!(f, "add {:?} {:?} {:?}", m, t, b),
            Sub(b, t, m) => write!(f, "sub {:?} {:?} {:?}", m, t, b),
            Mul(b, t, m) => write!(f, "mul {:?} {:?} {:?}", m, t, b),
            Div(b, t) => write!(f, "div {:?} {:?}", t, b),
            Mod(b, t) => write!(f, "mod {:?} {:?}", t, b),
            Shl(x, y, t) => write!(f, "shl {:?} {:?} {:?}", t, x, y),
            Shr(x, y, t) => write!(f, "shr {:?} {:?} {:?}", t, x, y),
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
            Ret(u, t) => write!(f, "ret {:?} {:?}", t, u),
            In(b) => write!(f, "in  {:?}", b),
            Out(u) => write!(f, "out {:?}", u),
            Fls => write!(f, "fls"),
            Sfd(x) => write!(f, "sfd {:?}", x),
            Gfd(x) => write!(f, "gfd {:?}", x),
            Zer(x, y) => write!(f, "zer {:?} {:?}", x, y),
            Cmp(x, y, z) => write!(f, "cmp {:?} {:?} {:?}", x, y, z),
            Cpy(x, y, z) => write!(f, "cpy {:?} {:?} {:?}", x, y, z),
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

    pub fn size(&self) -> UWord {
        use OpType::*;

        match self {
            U8 => std::mem::size_of::<u8>() as UWord,
            I8 => std::mem::size_of::<i8>() as UWord,
            U16 => std::mem::size_of::<u16>() as UWord,
            I16 => std::mem::size_of::<i16>() as UWord,
            U32 => std::mem::size_of::<u32>() as UWord,
            I32 => std::mem::size_of::<i32>() as UWord,
            U64 => std::mem::size_of::<u64>() as UWord,
            I64 => std::mem::size_of::<i64>() as UWord,
            Uw => std::mem::size_of::<UWord>() as UWord,
            Iw => std::mem::size_of::<IWord>() as UWord,
            F32 => std::mem::size_of::<f32>() as UWord,
            F64 => std::mem::size_of::<f64>() as UWord,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

    pub fn as_mode(&self) -> Mode { Mode(self.as_byte()) }
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

    /// Set zeroes mode.
    Zer,
}

impl ParameterMode {
    pub fn new(value: u8) -> Result<Self, UndefinedOperation> {
        use ParameterMode::*;

        Ok(match value {
            0 => Set,
            1 => Emp,
            2 => Zer,
            _ => return Err(UndefinedOperation::ParameterMode),
        })
    }

    pub fn as_byte(&self) -> u8 {
        use ParameterMode::*;

        match self {
            Set => 0,
            Emp => 1,
            Zer => 2,
        }
    }

    pub fn as_mode(&self) -> Mode { Mode(self.as_byte()) }
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
            Zer => write!(f, "zer"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    /// `x y` variant.
    None,

    /// `x{q} y` variant.
    First,

    /// `x y{q}` variant.
    Second,

    /// `x{q} y{w}` variant.
    Both,
}

impl Variant {
    pub fn new(variant: u8) -> Result<Self, UndefinedOperation> {
        use Variant::*;

        Ok(match variant {
            0 => None,
            1 => First,
            2 => Second,
            3 => Both,
            _ => return Err(UndefinedOperation::Variant),
        })
    }

    pub fn as_byte(&self) -> u8 {
        use Variant::*;

        match self {
            None => 0,
            First => 1,
            Second => 2,
            Both => 3,
        }
    }
}

impl Default for Variant {
    fn default() -> Self { Variant::None }
}
