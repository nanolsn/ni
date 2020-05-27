use super::{
    *,
    memory::*,
};

#[derive(Debug, Eq, PartialEq)]
pub enum ExecuteError {
    NotImplemented,
    NoProgram,
    IncorrectOperation,
}

#[derive(Debug)]
pub struct VM {
    code_ptr: usize,
    memory: Memory,
    ended: Option<usize>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            code_ptr: 0,
            memory: Memory::new(),
            ended: None,
        }
    }

    pub fn ended(&self) -> Option<usize> { self.ended }

    pub fn execute(&mut self) -> Result<(), ExecuteError> {
        use Op::*;

        // TODO: Get operation.
        let op = Nop;

        let res = match op {
            Nop => Ok(()),
            End(x) => {
                let code = x.get().ok_or(ExecuteError::IncorrectOperation)?;
                self.ended = Some(code);
                Ok(())
            }
            Slp(_) => Ok(()),
            _ => Err(ExecuteError::NotImplemented),
        };

        if res.is_ok() {
            self.code_ptr += 1;
        }

        res
    }
}
