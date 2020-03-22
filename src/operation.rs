#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Nop,
    Stop(Value),
    Wait(Value),
    Set(RefRet, Value, usize),
    Add(Ref, Value, Option<Ref>, usize),
    Sub(Ref, Value, Option<Ref>, usize),
    Mul(Ref, Value, Option<Ref>, usize),
    Div(Ref, Value, Option<Ref>, usize),
    Mod(Ref, Value, Option<Ref>, usize),
    Muls(Ref, Value, Option<Ref>, usize),
    Divs(Ref, Value, Option<Ref>, usize),
    Mods(Ref, Value, Option<Ref>, usize),
    Shl(Ref, Value, Option<Ref>, usize),
    Shr(Ref, Value, Option<Ref>, usize),
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
    fn from(f: u8) -> Self { Ref(f as usize) }
}
