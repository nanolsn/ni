use super::{
    *,
    memory::*,
};

#[derive(Debug)]
pub struct Function {
    frame_size: usize,
    program: Vec<Op>,
}

#[derive(Debug)]
pub struct StackFrame<'f> {
    function: &'f Function,
    base_address: usize,
    ret_address: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionError {
    NotImplemented,
    EndOfProgram,
    MemoryError(MemoryError),
    IncorrectOperation, // TODO: What the operation?
}

impl From<MemoryError> for ExecutionError {
    fn from(e: MemoryError) -> Self { ExecutionError::MemoryError(e) }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionSuccess {
    Ok,
    End(usize),
    Sleep(usize),
}

pub type Executed = Result<ExecutionSuccess, ExecutionError>;

#[derive(Debug)]
pub struct Executor<'f> {
    program_counter: usize,
    memory: Memory,
    functions: &'f [Function],
    stack: Vec<StackFrame<'f>>,
}

impl<'f> Executor<'f> {
    pub fn new(functions: &'f [Function]) -> Self {
        Self {
            program_counter: 0,
            memory: Memory::new(),
            functions,
            stack: Vec::new(),
        }
    }

    pub fn execute(&mut self) -> Executed {
        use Op::*;

        let op = todo!();

        let res = match op {
            Nop => Ok(ExecutionSuccess::Ok),
            End(x) => {
                let code = x.get().ok_or(ExecutionError::IncorrectOperation)?;

                // TODO: Destruct the operand.

                Ok(ExecutionSuccess::End(code))
            }
            Slp(x) => {
                let code = x.get().ok_or(ExecutionError::IncorrectOperation)?;

                // TODO: Destruct the operand.

                Ok(ExecutionSuccess::Sleep(code))
            }
            Set(_, _) => {
                todo!()
            }
            _ => Err(ExecutionError::NotImplemented),
        };

        if res.is_ok() {
            self.program_counter += 1;
        }

        res
    }
}
