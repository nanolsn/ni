#[derive(Debug, Eq, PartialEq)]
pub enum UndefinedOperation {
    OpType,
    Kind,
    Variant,
    ArithmeticMode,
    ParameterMode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
        match self {
            Operand::Loc(v) => Some(v),
            Operand::Ind(v) => Some(v),
            Operand::Ret(v) => Some(v),
            Operand::Val(v) => Some(v),
            Operand::Ref(v) => Some(v),
            Operand::Emp => None,
        }
    }

    pub fn map<F>(self, f: F) -> Self
        where
            F: FnOnce(usize) -> usize,
    {
        match self {
            Operand::Loc(v) => Operand::Loc(f(v)),
            Operand::Ind(v) => Operand::Ind(f(v)),
            Operand::Ret(v) => Operand::Ret(f(v)),
            Operand::Val(v) => Operand::Val(f(v)),
            Operand::Ref(v) => Operand::Ref(f(v)),
            Operand::Emp => Operand::Emp,
        }
    }
}

impl From<u8> for Operand {
    fn from(byte: u8) -> Self { Operand::Loc(byte as usize) }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    Cfn(Operand),
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
}

impl Default for ArithmeticMode {
    fn default() -> Self { ArithmeticMode::Wrap }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
}

impl Default for ParameterMode {
    fn default() -> Self { ParameterMode::Set }
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
