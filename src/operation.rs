#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Nop,
    Stop(Value),
    Wait(Value),
    Set(RefRet, Value, OpSize),
    Add(Ref, Value, Option<Ref>, OpSize),
    Sub(Ref, Value, Option<Ref>, OpSize),
    Mul(Ref, Value, Option<Ref>, OpSize),
    Div(Ref, Value, Option<Ref>, OpSize),
    Mod(Ref, Value, Option<Ref>, OpSize),
    Muls(Ref, Value, Option<Ref>, OpSize),
    Divs(Ref, Value, Option<Ref>, OpSize),
    Mods(Ref, Value, Option<Ref>, OpSize),
    Shl(Ref, Value, Option<Ref>, OpSize),
    Shr(Ref, Value, Option<Ref>, OpSize),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Ref(usize),
    Const(u64),
}

impl From<u8> for Value {
    fn from(f: u8) -> Self { Value::Ref(f as usize) }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RefRet {
    Ref(usize),
    Return(usize),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ref(pub usize);

impl From<u8> for Ref {
    fn from(b: u8) -> Self { Ref(b as usize) }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OpSize {
    B1,
    B2,
    B4,
    B8,
}

impl OpSize {
    pub fn size(self) -> usize {
        match self {
            OpSize::B1 => 1,
            OpSize::B2 => 2,
            OpSize::B4 => 4,
            OpSize::B8 => 8,
        }
    }
}

impl From<u8> for OpSize {
    fn from(b: u8) -> Self {
        match b {
            0 => OpSize::B1,
            1 => OpSize::B2,
            2 => OpSize::B4,
            3 => OpSize::B8,
            _ => panic!("Undefined OpSize"),
        }
    }
}

impl From<usize> for OpSize {
    fn from(size: usize) -> Self {
        match size {
            1 => OpSize::B1,
            2 => OpSize::B2,
            4 => OpSize::B4,
            8 => OpSize::B8,
            _ => panic!("Undefined OpSize"),
        }
    }
}
