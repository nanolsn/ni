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
pub struct FunctionCall<'f> {
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
    call_stack: Vec<FunctionCall<'f>>,
}

impl<'f> Executor<'f> {
    pub fn new(functions: &'f [Function]) -> Self {
        Self {
            functions,
            memory: Memory::new(),
            program_counter: 0,
            call_stack: Vec::new(),
        }
    }

    pub fn call(&mut self, function_id: usize, ret_address: usize) -> Result<(), ExecutionError> {
        let f = self.functions.get(function_id).ok_or(ExecutionError::UnknownFunction)?;
        self.call_stack.push(FunctionCall {
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
        let top_fn = self.call_stack.pop().ok_or(ExecutionError::EndOfProgram)?;

        self.program_counter = top_fn.ret_program_counter;
        self.memory.stack.narrow(top_fn.function.frame_size)?;

        Ok(())
    }

    fn current_function(&self) -> Result<&FunctionCall, ExecutionError> {
        self.call_stack.last().ok_or(ExecutionError::EndOfProgram)
    }

    fn get_val<T>(&self, operand: Operand) -> Result<T, ExecutionError>
        where
            T: Primary,
    {
        Ok(match operand {
            Operand::Loc(loc) => self.memory.get(
                self.current_function()?.base_address.wrapping_add(loc)
            )?,
            Operand::Ind(ptr) => self.memory.get(
                self.memory.get(self.current_function()?.base_address.wrapping_add(ptr))?
            )?,
            Operand::Ret(ret) => self.memory.get(
                self.current_function()?.ret_address.wrapping_add(ret)
            )?,
            Operand::Val(val) => T::from_usize(val),
            Operand::Ref(var) => T::from_usize(
                self.current_function()?.base_address.wrapping_add(var)
            ),
            Operand::Emp => return Err(ExecutionError::IncorrectOperation),
        })
    }

    fn set_val<T>(&mut self, operand: Operand, val: T) -> Result<(), ExecutionError>
        where
            T: Primary,
    {
        Ok(match operand {
            Operand::Loc(loc) => self.memory.set(
                self.current_function()?.base_address.wrapping_add(loc),
                val,
            )?,
            Operand::Ind(ptr) => self.memory.set(
                self.memory.get(self.current_function()?.base_address.wrapping_add(ptr))?,
                val,
            )?,
            Operand::Ret(ret) => self.memory.set(
                self.current_function()?.ret_address.wrapping_add(ret),
                val,
            )?,
            Operand::Val(_) => return Err(ExecutionError::IncorrectOperation),
            Operand::Ref(_) => return Err(ExecutionError::IncorrectOperation),
            Operand::Emp => return Err(ExecutionError::IncorrectOperation),
        })
    }

    fn update_un<T, F>(&mut self, un: UnOp, f: F) -> Result<(), ExecutionError>
        where
            T: Primary,
            F: FnOnce(T) -> T,
    {
        let left = if let Some(offset) = un.x_offset {
            self.make_offset(un.x, offset)?
        } else {
            un.x
        };

        self.set_val(left, f(self.get_val(left)?))
    }

    fn update_bin<T, F>(&mut self, bin: BinOp, f: F) -> Result<(), ExecutionError>
        where
            T: Primary,
            F: FnOnce(T, T) -> T,
    {
        let left = if let Some(offset) = bin.x_offset {
            self.make_offset(bin.x, offset)?
        } else {
            bin.x
        };

        let right = if let Some(offset) = bin.y_offset {
            self.make_offset(bin.y, offset)?
        } else {
            bin.y
        };

        self.set_val(left, f(self.get_val(left)?, self.get_val(right)?))
    }

    fn make_offset(&self, a: Operand, offset: Operand) -> Result<Operand, ExecutionError> {
        let a_offset: usize = self.get_val(offset)?;
        Ok(a.map(|a| a.wrapping_add(a_offset)))
    }

    pub fn execute(&mut self) -> Executed {
        use Op::*;

        let &op = self.current_function()?.function.program
            .get(self.program_counter)
            .ok_or(ExecutionError::EndOfProgram)?;

        let res = match op {
            Nop => Ok(ExecutionSuccess::Ok),
            End(x) => {
                let val = self.get_val(x)?;
                Ok(ExecutionSuccess::End(val))
            }
            Slp(x) => {
                let val = self.get_val(x)?;
                Ok(ExecutionSuccess::Sleep(val))
            }
            Set(bo, ot) => {
                match ot {
                    OpType::U8 => self.update_bin::<u8, _>(bo, |_, y| y)?,
                    OpType::I8 => self.update_bin::<i8, _>(bo, |_, y| y)?,
                    OpType::U16 => self.update_bin::<u16, _>(bo, |_, y| y)?,
                    OpType::I16 => self.update_bin::<i16, _>(bo, |_, y| y)?,
                    OpType::U32 => self.update_bin::<u32, _>(bo, |_, y| y)?,
                    OpType::I32 => self.update_bin::<i32, _>(bo, |_, y| y)?,
                    OpType::U64 => self.update_bin::<u64, _>(bo, |_, y| y)?,
                    OpType::I64 => self.update_bin::<i64, _>(bo, |_, y| y)?,
                    OpType::Uw => self.update_bin::<usize, _>(bo, |_, y| y)?,
                    OpType::Iw => self.update_bin::<isize, _>(bo, |_, y| y)?,
                    OpType::F32 => self.update_bin::<f32, _>(bo, |_, y| y)?,
                    OpType::F64 => self.update_bin::<f64, _>(bo, |_, y| y)?,
                }

                Ok(ExecutionSuccess::Ok)
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
    fn executor_set_get_val() {
        let functions = [
            Function {
                frame_size: 8,
                program: vec![],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();
        exe.call(0, 0).unwrap();

        assert_eq!(exe.set_val(Operand::Loc(0), 8), Ok(()));
        assert_eq!(exe.get_val::<usize>(Operand::Loc(0)), Ok(8));

        assert_eq!(exe.set_val(Operand::Ind(0), 8), Ok(()));
        assert_eq!(exe.get_val::<usize>(Operand::Ind(0)), Ok(8));

        assert_eq!(exe.set_val(Operand::Ret(0), 3), Ok(()));
        assert_eq!(exe.get_val::<usize>(Operand::Ret(0)), Ok(3));

        assert_eq!(exe.set_val(Operand::Val(7), 0), Err(ExecutionError::IncorrectOperation));
        assert_eq!(exe.get_val::<usize>(Operand::Val(8)), Ok(8));

        assert_eq!(exe.set_val(Operand::Ref(0), 0), Err(ExecutionError::IncorrectOperation));
        assert_eq!(exe.get_val::<usize>(Operand::Ref(0)), Ok(8));

        assert_eq!(exe.set_val(Operand::Emp, 0), Err(ExecutionError::IncorrectOperation));
        assert_eq!(exe.get_val::<usize>(Operand::Emp), Err(ExecutionError::IncorrectOperation));
    }

    #[test]
    fn executor_set() {
        let fb = f32::to_le_bytes(0.123);
        let float = usize::from_le_bytes([fb[0], fb[1], fb[2], fb[3], 0, 0, 0, 0]);

        let functions = [
            Function {
                frame_size: 4,
                program: vec![
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(12)), OpType::I32),
                    Op::Set(BinOp::new(Operand::Val(0), Operand::Val(12)), OpType::I32),
                    Op::Set(BinOp::new(Operand::Emp, Operand::Val(12)), OpType::I32),
                    Op::Set(BinOp::new(Operand::Loc(1), Operand::Val(32)), OpType::I8),
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(float)), OpType::F32),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(0)), Ok(12));

        assert_eq!(exe.execute(), Executed::Err(ExecutionError::IncorrectOperation));
        exe.program_counter += 1; // Move manually after incorrect operation

        assert_eq!(exe.execute(), Executed::Err(ExecutionError::IncorrectOperation));
        exe.program_counter += 1; // Move manually after incorrect operation

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i8>(Operand::Loc(1)), Ok(32));

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<f32>(Operand::Loc(0)), Ok(0.123));
    }
}
