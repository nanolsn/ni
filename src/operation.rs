#[derive(Copy, Clone, Debug)]
pub enum Op {
    Nop,
    Stop(Value<u64>),
    Wait(Value<u64>),
    Set(RetValue, Value<u64>, OpSize),
}

#[derive(Copy, Clone, Debug)]
pub enum Value<T> {
    Ref(u32),
    Const(T),
}

#[derive(Copy, Clone, Debug)]
pub enum RetValue {
    Ref(u32),
    Ret(u32),
}

#[derive(Copy, Clone, Debug)]
pub enum OpSize {
    B1,
    B2,
    B4,
    B8,
}
