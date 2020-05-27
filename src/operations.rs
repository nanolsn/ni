#[derive(Debug, Eq, PartialEq)]
pub enum UndefinedOperation {
    OpType,
    Kind,
    Variant,
    ArithmeticMode,
    ParameterMode,
}

#[derive(Debug, Eq, PartialEq)]
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
    Ift(Operand),
    Iff(Operand),
    Ife(BinOp, OpType),
    Ifl(BinOp, OpType),
    Ifg(BinOp, OpType),
    Ine(BinOp, OpType),
    Inl(BinOp, OpType),
    Ing(BinOp, OpType),
    Ifa(Operand, Operand),
    Ifo(Operand, Operand),
    Ifx(Operand, Operand),
    Ina(Operand, Operand),
    Ino(Operand, Operand),
    Inx(Operand, Operand),
    App(Operand),
    Par(UnOp, OpType, ParameterMode),
    Cfn(Operand),
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

#[derive(Debug, Eq, PartialEq)]
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

#[derive(Debug, Eq, PartialEq)]
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
