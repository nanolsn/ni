#[derive(Copy, Clone, Debug)]
pub enum Op {
    Nop,
    Stop(u64),
}
