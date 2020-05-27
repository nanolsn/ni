use super::*;

#[derive(Debug, Eq, PartialEq)]
pub enum ExecuteError {
    NotImplemented,
    NoProgram,
    IncorrectOperation,
}

#[derive(Clone, Debug)]
pub struct Function {
    stackframe_size: usize,
    program: Vec<Op>,
}

#[derive(Clone, Debug)]
pub struct Frame {
    function: Function,
    operation_ptr: usize,
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<Frame>,
    functions: Vec<Function>,
    ended: Option<usize>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            functions: Vec::new(),
            ended: None,
        }
    }

    pub fn ended(&self) -> Option<usize> { self.ended }

    pub fn add_fn(&mut self, f: Function) { self.functions.push(f) }

    pub fn run(&mut self, f: usize) {
        self.stack.push(Frame {
            function: self.functions[f].clone(),
            operation_ptr: 0,
        })
    }

    pub fn execute(&mut self) -> Result<(), ExecuteError> {
        use Op::*;

        let frame = self.stack.last_mut().ok_or(ExecuteError::NoProgram)?;
        let op = &frame.function.program[frame.operation_ptr];

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
            frame.operation_ptr += 1;
        }

        res
    }
}
