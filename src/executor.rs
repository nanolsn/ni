use super::{
    *,
    memory::*,
};

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
pub struct VM {
    code_ptr: usize,
    memory: Memory,
    program: Vec<Op>,
}

impl VM {
    pub fn new(program: Vec<Op>) -> Self {
        Self {
            code_ptr: 0,
            memory: Memory::new(),
            program,
        }
    }

    pub fn execute(&mut self) -> Executed {
        use Op::*;

        let op = self.program.get(self.code_ptr).ok_or(ExecutionError::EndOfProgram)?;

        let res = match op {
            Nop => Ok(ExecutionSuccess::Ok),
            End(x) => {
                let code = x.get().ok_or(ExecutionError::IncorrectOperation)?;
                Ok(ExecutionSuccess::End(code))
            }
            Slp(x) => {
                let code = x.get().ok_or(ExecutionError::IncorrectOperation)?;
                Ok(ExecutionSuccess::Sleep(code))
            }
            Set(_, _) => {
                todo!()
            }
            _ => Err(ExecutionError::NotImplemented),
        };

        if res.is_ok() {
            self.code_ptr += 1;
        }

        res
    }
}
