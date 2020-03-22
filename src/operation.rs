#[derive(Copy, Clone, Debug)]
pub enum Op {
    Nop,
    Stop(Value),
    Wait(Value),
    Set(RetValue, Value, usize),
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Ref(usize),
    Const(u64),
}

#[derive(Copy, Clone, Debug)]
pub enum RetValue {
    Ref(usize),
    Return(usize),
}
