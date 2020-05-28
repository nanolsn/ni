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
    OperationOverflow,
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
            ret_program_counter: self.program_counter.wrapping_add(1),
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

    fn update_un<T, U, F>(&mut self, un: UnOp, f: F) -> Result<(), ExecutionError>
        where
            T: Primary,
            U: Primary,
            F: FnOnce(T) -> U,
    {
        let left = if let Some(offset) = un.x_offset {
            self.make_offset(un.x, offset)?
        } else {
            un.x
        };

        self.set_val(left, f(self.get_val(left)?))
    }

    fn update_bin<T, U, F>(&mut self, bin: BinOp, f: F) -> Result<(), ExecutionError>
        where
            T: Primary,
            U: Primary,
            F: FnOnce(T, T) -> U,
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

    fn set<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Primary,
    { self.update_bin::<T, T, _>(bin, |_, y| y) }

    fn add<T, U>(&mut self, bin: BinOp, mode: ArithmeticMode) -> Result<(), ExecutionError>
        where
            T: Add,
            U: Add + From<T>,
    {
        use ArithmeticMode::*;

        match mode {
            Wrap => self.update_bin::<T, T, _>(bin, |x, y| x.wrapping(y)),
            Sat => self.update_bin::<T, T, _>(bin, |x, y| x.saturating(y)),
            Wide => self.update_bin::<T, U, _>(
                bin,
                |x, y| U::from(x).wrapping(U::from(y)),
            ),
            Hand => {
                let mut overflowed = false;

                self.update_bin::<T, T, _>(bin, |x, y| {
                    if let Some(s) = x.checked(y) {
                        s
                    } else {
                        overflowed = true;
                        <T as Primary>::zero()
                    }
                })?;

                if overflowed {
                    Err(ExecutionError::OperationOverflow)
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn execute(&mut self) -> Executed {
        use Op::*;
        use OpType::*;

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
            Set(bin, ot) => {
                match ot {
                    U8 => self.set::<u8>(bin)?,
                    I8 => self.set::<i8>(bin)?,
                    U16 => self.set::<u16>(bin)?,
                    I16 => self.set::<i16>(bin)?,
                    U32 => self.set::<u32>(bin)?,
                    I32 => self.set::<i32>(bin)?,
                    U64 => self.set::<u64>(bin)?,
                    I64 => self.set::<i64>(bin)?,
                    Uw => self.set::<usize>(bin)?,
                    Iw => self.set::<isize>(bin)?,
                    F32 => self.set::<f32>(bin)?,
                    F64 => self.set::<f64>(bin)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Add(bin, ot, mode) => {
                match ot {
                    U8 => self.add::<u8, u16>(bin, mode)?,
                    I8 => self.add::<i8, i16>(bin, mode)?,
                    U16 => self.add::<u16, u32>(bin, mode)?,
                    I16 => self.add::<i16, i32>(bin, mode)?,
                    U32 => self.add::<u32, u64>(bin, mode)?,
                    I32 => self.add::<i32, i64>(bin, mode)?,
                    U64 => self.add::<u64, u128>(bin, mode)?,
                    I64 => self.add::<i64, i128>(bin, mode)?,
                    Uw => self.add::<usize, usize>(bin, mode)?,
                    Iw => self.add::<isize, isize>(bin, mode)?,
                    F32 => self.add::<f32, f32>(bin, mode)?,
                    F64 => self.add::<f64, f64>(bin, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            _ => Err(ExecutionError::NotImplemented),
        };

        if res.is_ok() {
            self.program_counter = self.program_counter.wrapping_add(1);
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

    #[test]
    fn executor_add() {
        let functions = [
            Function {
                frame_size: 8,
                program: vec![
                    Op::Add(
                        BinOp::new(Operand::Loc(0), Operand::Val(12)),
                        OpType::I32,
                        ArithmeticMode::Wrap,
                    ),
                    Op::Add(
                        BinOp::new(Operand::Loc(0), Operand::Val(u32::MAX as usize)),
                        OpType::I32,
                        ArithmeticMode::Wrap,
                    ),
                    Op::Add(
                        BinOp::new(Operand::Loc(0), Operand::Val(i32::MAX as usize)),
                        OpType::I32,
                        ArithmeticMode::Sat,
                    ),
                    Op::Add(
                        BinOp::new(Operand::Loc(0), Operand::Val(i32::MAX as usize)),
                        OpType::I32,
                        ArithmeticMode::Wide,
                    ),
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(1)), OpType::I32),
                    Op::Add(
                        BinOp::new(Operand::Loc(0), Operand::Val(i32::MAX as usize)),
                        OpType::I32,
                        ArithmeticMode::Hand,
                    ),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(0)), Ok(12));

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(0)), Ok(11));

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(0)), Ok(i32::MAX));

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i64>(Operand::Loc(0)), Ok(i32::MAX as i64 * 2));

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Err(ExecutionError::OperationOverflow));
    }
}
