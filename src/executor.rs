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
    ret_program_counter: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionError {
    NotImplemented,
    EndOfProgram,
    MemoryError(MemoryError),

    // TODO: What the operation?
    IncorrectOperation,
    UnknownFunction,
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
    functions: &'f [Function],
    memory: Memory,
    program_counter: usize,
    stack: Vec<StackFrame<'f>>,
}

impl<'f> Executor<'f> {
    pub fn new(functions: &'f [Function]) -> Self {
        Self {
            functions,
            memory: Memory::new(),
            program_counter: 0,
            stack: Vec::new(),
        }
    }

    pub fn call(&mut self, function_id: usize, ret_address: usize) -> Result<(), ExecutionError> {
        let f = self.functions.get(function_id).ok_or(ExecutionError::UnknownFunction)?;
        self.stack.push(StackFrame {
            function: f,
            base_address: self.memory.stack.len(),
            ret_address,
            ret_program_counter: self.program_counter + 1,
        });

        self.program_counter = 0;
        self.memory.stack.expand(f.frame_size)?;

        Ok(())
    }

    fn ret(&mut self) -> Result<(), ExecutionError> {
        let stackframe = self.stack.pop().ok_or(ExecutionError::EndOfProgram)?;

        self.program_counter = stackframe.ret_program_counter;
        self.memory.stack.narrow(stackframe.function.frame_size)?;

        Ok(())
    }

    pub fn execute(&mut self) -> Executed {
        use Op::*;

        let stackframe = self.stack.last().ok_or(ExecutionError::EndOfProgram)?;
        let &op = stackframe.function.program
            .get(self.program_counter)
            .ok_or(ExecutionError::EndOfProgram)?;

        let res = match op {
            Nop => Ok(ExecutionSuccess::Ok),
            End(x) => {
                let val = match x {
                    Operand::Loc(loc) => self.memory.get(
                        stackframe.base_address.wrapping_add(loc)
                    )?,
                    Operand::Ind(ptr) => self.memory.get(
                        self.memory.get(stackframe.base_address.wrapping_add(ptr))?
                    )?,
                    Operand::Ret(ret) => self.memory.get(
                        stackframe.ret_address.wrapping_add(ret)
                    )?,
                    Operand::Val(val) => val,
                    Operand::Ref(var) => stackframe.base_address.wrapping_add(var),
                    Operand::Emp => return Err(ExecutionError::IncorrectOperation),
                };

                Ok(ExecutionSuccess::End(val))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor() {
        let functions = [
            Function {
                frame_size: 8,
                program: vec![
                    Op::End(Operand::Val(12)),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::End(12)));
    }
}
