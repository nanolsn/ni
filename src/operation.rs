use crate::operation::OpSize::B1;

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
    Return(u32),
}

#[derive(Copy, Clone, Debug)]
pub enum OpSize {
    B1,
    B2,
    B4,
    B8,
}

impl OpSize {
    pub fn new(size: usize) -> Self {
        match size {
            1 => OpSize::B1,
            2 => OpSize::B2,
            4 => OpSize::B4,
            8 => OpSize::B8,
            _ => panic!("Undefined OpSize"),
        }
    }
}

impl From<usize> for OpSize {
    fn from(size: usize) -> Self { OpSize::new(size) }
}
