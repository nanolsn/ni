use common::*;
use super::{
    memory::*,
    primary::*,
};

#[derive(Debug)]
pub struct Function<'f> {
    frame_size: usize,
    program: &'f [Op],
}

#[derive(Debug)]
pub struct FunctionCall<'f> {
    function: &'f Function<'f>,
    base_address: usize,
    ret_address: usize,
    ret_program_counter: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionError {
    NotImplemented,
    EndOfProgram,
    MemoryError(MemoryError),
    IncorrectOperation(Op),
    UnknownFunction,
    OperationOverflow,
    DivisionByZero,
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
    functions: &'f [Function<'f>],
    memory: Memory,
    program_counter: usize,
    call_stack: Vec<FunctionCall<'f>>,
    prepared_call: bool,
    parameter_ptr: usize,
}

macro_rules! impl_exec_bin {
    ($name:ident, $tr:ident) => {
        fn $name<T: $tr, U: $tr + From<T>>(&mut self, bin: BinOp, mode: ArithmeticMode)
            -> Result<(), ExecutionError> {
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
    };
}

macro_rules! impl_exec_un {
    ($name:ident, $tr:ident) => {
        fn $name<T: $tr, U: $tr + From<T>>(&mut self, un: UnOp, mode: ArithmeticMode)
            -> Result<(), ExecutionError> {
            use ArithmeticMode::*;

            match mode {
                Wrap => self.update_un::<T, T, _>(un, |x| x.wrapping()),
                Sat => self.update_un::<T, T, _>(un, |x| x.saturating()),
                Wide => self.update_un::<T, U, _>(un, |x| U::from(x).wrapping()),
                Hand => {
                    let mut overflowed = false;

                    self.update_un::<T, T, _>(un, |x| {
                        if let Some(s) = x.checked() {
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
    };
}

impl<'f> Executor<'f> {
    pub fn new(functions: &'f [Function]) -> Self {
        Self {
            functions,
            memory: Memory::new(),
            program_counter: 0,
            call_stack: Vec::new(),
            prepared_call: false,
            parameter_ptr: 0,
        }
    }

    fn app(&mut self, function_id: usize) -> Result<(), ExecutionError> {
        let f = self.functions.get(function_id).ok_or(ExecutionError::UnknownFunction)?;
        self.call_stack.push(FunctionCall {
            function: f,
            base_address: self.memory.stack.len(),
            ret_address: 0,
            ret_program_counter: 0,
        });

        self.prepared_call = true;
        self.memory.stack.expand(f.frame_size)?;

        Ok(())
    }

    fn clf(&mut self, ret_address: usize) -> Result<(), ExecutionError> {
        let current_fn = self.call_stack
            .last_mut()
            .ok_or(ExecutionError::EndOfProgram)?;

        current_fn.ret_address = ret_address;
        current_fn.ret_program_counter = self.program_counter.wrapping_add(1);
        self.prepared_call = false;
        self.program_counter = 0;
        self.parameter_ptr = 0;

        Ok(())
    }

    pub fn call(&mut self, function_id: usize, ret_address: usize) -> Result<(), ExecutionError> {
        self.app(function_id)?;
        self.clf(ret_address)
    }

    fn ret(&mut self) -> Result<(), ExecutionError> {
        let current_fn = self.call_stack.pop().ok_or(ExecutionError::EndOfProgram)?;

        self.program_counter = current_fn.ret_program_counter;
        self.memory.stack.narrow(current_fn.function.frame_size)?;

        Ok(())
    }

    fn current_call(&self) -> Result<&FunctionCall, ExecutionError> {
        let call = if self.prepared_call {
            self.call_stack.get(self.call_stack.len().wrapping_sub(2))
        } else {
            self.call_stack.last()
        };

        call.ok_or(ExecutionError::EndOfProgram)
    }

    fn current_op(&self) -> Result<&Op, ExecutionError> {
        self.current_call()?.function.program
            .get(self.program_counter)
            .ok_or(ExecutionError::EndOfProgram)
    }

    fn pass_condition(&mut self) -> Result<(), ExecutionError> {
        loop {
            self.program_counter += 1;
            let op = self.current_op()?;

            if !op.is_conditional() {
                self.program_counter += 1;
                break Ok(());
            }
        }
    }

    fn get_val<T>(&self, operand: Operand) -> Result<T, ExecutionError>
        where
            T: Primary,
    {
        Ok(match operand {
            Operand::Loc(loc) => self.memory.get(
                self.current_call()?.base_address.wrapping_add(loc)
            )?,
            Operand::Ind(ptr) => self.memory.get(
                self.memory.get(self.current_call()?.base_address.wrapping_add(ptr))?
            )?,
            Operand::Ret(ret) => self.memory.get(
                self.current_call()?.ret_address.wrapping_add(ret)
            )?,
            Operand::Val(val) => T::from_usize(val),
            Operand::Ref(var) => T::from_usize(
                self.current_call()?.base_address.wrapping_add(var)
            ),
            Operand::Emp => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
        })
    }

    fn set_val<T>(&mut self, operand: Operand, val: T) -> Result<(), ExecutionError>
        where
            T: Primary,
    {
        Ok(match operand {
            Operand::Loc(loc) => self.memory.set(
                self.current_call()?.base_address.wrapping_add(loc),
                val,
            )?,
            Operand::Ind(ptr) => self.memory.set(
                self.memory.get(self.current_call()?.base_address.wrapping_add(ptr))?,
                val,
            )?,
            Operand::Ret(ret) => self.memory.set(
                self.current_call()?.ret_address.wrapping_add(ret),
                val,
            )?,
            Operand::Val(_) => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
            Operand::Ref(_) => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
            Operand::Emp => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
        })
    }

    fn read_un_operand(&self, un: UnOp) -> Result<Operand, ExecutionError> {
        let left = if let Some(offset) = un.x_offset {
            self.make_offset(un.x, offset)?
        } else {
            un.x
        };

        Ok(left)
    }

    fn read_bin_operands(&self, bin: BinOp) -> Result<(Operand, Operand), ExecutionError> {
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

        Ok((left, right))
    }

    fn get_un<T>(&mut self, un: UnOp) -> Result<T, ExecutionError>
        where
            T: Primary,
    {
        let left = self.read_un_operand(un)?;
        self.get_val(left)
    }

    fn update_un<T, U, F>(&mut self, un: UnOp, f: F) -> Result<(), ExecutionError>
        where
            T: Primary,
            U: Primary,
            F: FnOnce(T) -> U,
    {
        let left = self.read_un_operand(un)?;
        self.set_val(left, f(self.get_val(left)?))
    }

    fn update_bin<T, U, F>(&mut self, bin: BinOp, f: F) -> Result<(), ExecutionError>
        where
            T: Primary,
            U: Primary,
            F: FnOnce(T, T) -> U,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        self.set_val(left, f(self.get_val(left)?, self.get_val(right)?))
    }

    fn update_bin_division<T, F>(&mut self, bin: BinOp, f: F) -> Result<(), ExecutionError>
        where
            T: Primary + PartialEq,
            F: FnOnce(T, T) -> T,
    {
        let mut div_by_zero = false;

        let res = self.update_bin::<T, T, _>(bin, |x, y| {
            if y == T::zero() {
                div_by_zero = true;
                T::zero()
            } else {
                f(x, y)
            }
        });

        if div_by_zero {
            return Err(ExecutionError::DivisionByZero);
        }

        res
    }

    fn make_offset(&self, a: Operand, offset: Operand) -> Result<Operand, ExecutionError> {
        let a_offset: usize = self.get_val(offset)?;
        Ok(a.map(|a| a.wrapping_add(a_offset)))
    }

    fn exec_set<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Primary,
    { self.update_bin::<T, T, _>(bin, |_, y| y) }

    impl_exec_bin!(exec_add, Add);
    impl_exec_bin!(exec_sub, Sub);
    impl_exec_bin!(exec_mul, Mul);

    fn exec_div<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Div + PartialEq,
    { self.update_bin_division::<T, _>(bin, |x, y| x.wrapping(y)) }

    fn exec_mod<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Rem + PartialEq,
    { self.update_bin_division::<T, _>(bin, |x, y| x.wrapping(y)) }

    impl_exec_bin!(exec_shl, Shl);
    impl_exec_bin!(exec_shr, Shr);

    fn exec_and<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Primary + std::ops::BitAnd<Output=T>,
    { self.update_bin::<T, T, _>(bin, |x, y| x & y) }

    fn exec_or<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Primary + std::ops::BitOr<Output=T>,
    { self.update_bin::<T, T, _>(bin, |x, y| x | y) }

    fn exec_xor<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Primary + std::ops::BitXor<Output=T>,
    { self.update_bin::<T, T, _>(bin, |x, y| x ^ y) }

    fn exec_not<T>(&mut self, un: UnOp) -> Result<(), ExecutionError>
        where
            T: Primary + std::ops::Not<Output=T>,
    { self.update_un::<T, T, _>(un, |y| !y) }

    impl_exec_un!(exec_neg, Neg);
    impl_exec_un!(exec_inc, Inc);
    impl_exec_un!(exec_dec, Dec);

    fn exec_ife<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? == self.get_val::<T>(right)?)
    }

    fn exec_ifl<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialOrd,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? < self.get_val::<T>(right)?)
    }

    fn exec_ifg<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialOrd,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? > self.get_val::<T>(right)?)
    }

    fn exec_ine<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? != self.get_val::<T>(right)?)
    }

    fn exec_inl<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialOrd,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? >= self.get_val::<T>(right)?)
    }

    fn exec_ing<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialOrd,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? <= self.get_val::<T>(right)?)
    }

    fn exec_ifa<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq + std::ops::BitAnd<Output=T>,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? & self.get_val::<T>(right)? != T::zero())
    }

    fn exec_ifo<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq + std::ops::BitOr<Output=T>,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? | self.get_val::<T>(right)? != T::zero())
    }

    fn exec_ifx<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq + std::ops::BitXor<Output=T>,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? ^ self.get_val::<T>(right)? != T::zero())
    }

    fn exec_ina<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq + std::ops::BitAnd<Output=T>,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? & self.get_val::<T>(right)? == T::zero())
    }

    fn exec_ino<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq + std::ops::BitOr<Output=T>,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? | self.get_val::<T>(right)? == T::zero())
    }

    fn exec_inx<T>(&self, bin: BinOp) -> Result<bool, ExecutionError>
        where
            T: Primary + PartialEq + std::ops::BitXor<Output=T>,
    {
        let (left, right) = self.read_bin_operands(bin)?;
        Ok(self.get_val::<T>(left)? ^ self.get_val::<T>(right)? == T::zero())
    }

    fn exec_par<T>(&mut self, un: UnOp, mode: ParameterMode) -> Result<(), ExecutionError>
        where
            T: Primary,
    {
        let frame_size = self.current_call()?.function.frame_size;
        let parameter_loc = self.parameter_ptr.wrapping_add(frame_size);
        self.parameter_ptr = self.parameter_ptr.wrapping_add(T::SIZE);

        match mode {
            ParameterMode::Set => {
                let val = self.get_un(un)?;
                self.set_val::<T>(Operand::Loc(parameter_loc), val)?;
            }
            ParameterMode::Emp => {}
            ParameterMode::Msz => {
                self.set_val::<T>(Operand::Loc(parameter_loc), T::zero())?;
            }
        }

        Ok(())
    }

    pub fn execute(&mut self) -> Executed {
        use Op::*;
        use OpType::*;

        let &op = self.current_op()?;

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
                    U8 => self.exec_set::<u8>(bin)?,
                    I8 => self.exec_set::<i8>(bin)?,
                    U16 => self.exec_set::<u16>(bin)?,
                    I16 => self.exec_set::<i16>(bin)?,
                    U32 => self.exec_set::<u32>(bin)?,
                    I32 => self.exec_set::<i32>(bin)?,
                    U64 => self.exec_set::<u64>(bin)?,
                    I64 => self.exec_set::<i64>(bin)?,
                    Uw => self.exec_set::<usize>(bin)?,
                    Iw => self.exec_set::<isize>(bin)?,
                    F32 => self.exec_set::<f32>(bin)?,
                    F64 => self.exec_set::<f64>(bin)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Add(bin, ot, mode) => {
                match ot {
                    U8 => self.exec_add::<u8, u16>(bin, mode)?,
                    I8 => self.exec_add::<i8, i16>(bin, mode)?,
                    U16 => self.exec_add::<u16, u32>(bin, mode)?,
                    I16 => self.exec_add::<i16, i32>(bin, mode)?,
                    U32 => self.exec_add::<u32, u64>(bin, mode)?,
                    I32 => self.exec_add::<i32, i64>(bin, mode)?,
                    U64 => self.exec_add::<u64, u128>(bin, mode)?,
                    I64 => self.exec_add::<i64, i128>(bin, mode)?,
                    Uw => self.exec_add::<usize, usize>(bin, mode)?,
                    Iw => self.exec_add::<isize, isize>(bin, mode)?,
                    F32 => self.exec_add::<f32, f32>(bin, mode)?,
                    F64 => self.exec_add::<f64, f64>(bin, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Sub(bin, ot, mode) => {
                match ot {
                    U8 => self.exec_sub::<u8, u16>(bin, mode)?,
                    I8 => self.exec_sub::<i8, i16>(bin, mode)?,
                    U16 => self.exec_sub::<u16, u32>(bin, mode)?,
                    I16 => self.exec_sub::<i16, i32>(bin, mode)?,
                    U32 => self.exec_sub::<u32, u64>(bin, mode)?,
                    I32 => self.exec_sub::<i32, i64>(bin, mode)?,
                    U64 => self.exec_sub::<u64, u128>(bin, mode)?,
                    I64 => self.exec_sub::<i64, i128>(bin, mode)?,
                    Uw => self.exec_sub::<usize, usize>(bin, mode)?,
                    Iw => self.exec_sub::<isize, isize>(bin, mode)?,
                    F32 => self.exec_sub::<f32, f32>(bin, mode)?,
                    F64 => self.exec_sub::<f64, f64>(bin, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Mul(bin, ot, mode) => {
                match ot {
                    U8 => self.exec_mul::<u8, u16>(bin, mode)?,
                    I8 => self.exec_mul::<i8, i16>(bin, mode)?,
                    U16 => self.exec_mul::<u16, u32>(bin, mode)?,
                    I16 => self.exec_mul::<i16, i32>(bin, mode)?,
                    U32 => self.exec_mul::<u32, u64>(bin, mode)?,
                    I32 => self.exec_mul::<i32, i64>(bin, mode)?,
                    U64 => self.exec_mul::<u64, u128>(bin, mode)?,
                    I64 => self.exec_mul::<i64, i128>(bin, mode)?,
                    Uw => self.exec_mul::<usize, usize>(bin, mode)?,
                    Iw => self.exec_mul::<isize, isize>(bin, mode)?,
                    F32 => self.exec_mul::<f32, f32>(bin, mode)?,
                    F64 => self.exec_mul::<f64, f64>(bin, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Div(bin, ot) => {
                match ot {
                    U8 => self.exec_div::<u8>(bin)?,
                    I8 => self.exec_div::<i8>(bin)?,
                    U16 => self.exec_div::<u16>(bin)?,
                    I16 => self.exec_div::<i16>(bin)?,
                    U32 => self.exec_div::<u32>(bin)?,
                    I32 => self.exec_div::<i32>(bin)?,
                    U64 => self.exec_div::<u64>(bin)?,
                    I64 => self.exec_div::<i64>(bin)?,
                    Uw => self.exec_div::<usize>(bin)?,
                    Iw => self.exec_div::<isize>(bin)?,
                    F32 => self.exec_div::<f32>(bin)?,
                    F64 => self.exec_div::<f64>(bin)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Mod(bin, ot) => {
                match ot {
                    U8 => self.exec_mod::<u8>(bin)?,
                    I8 => self.exec_mod::<i8>(bin)?,
                    U16 => self.exec_mod::<u16>(bin)?,
                    I16 => self.exec_mod::<i16>(bin)?,
                    U32 => self.exec_mod::<u32>(bin)?,
                    I32 => self.exec_mod::<i32>(bin)?,
                    U64 => self.exec_mod::<u64>(bin)?,
                    I64 => self.exec_mod::<i64>(bin)?,
                    Uw => self.exec_mod::<usize>(bin)?,
                    Iw => self.exec_mod::<isize>(bin)?,
                    F32 => self.exec_mod::<f32>(bin)?,
                    F64 => self.exec_mod::<f64>(bin)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Shl(bin, ot, mode) => {
                match ot {
                    U8 => self.exec_shl::<u8, u16>(bin, mode)?,
                    I8 => self.exec_shl::<i8, i16>(bin, mode)?,
                    U16 => self.exec_shl::<u16, u32>(bin, mode)?,
                    I16 => self.exec_shl::<i16, i32>(bin, mode)?,
                    U32 => self.exec_shl::<u32, u64>(bin, mode)?,
                    I32 => self.exec_shl::<i32, i64>(bin, mode)?,
                    U64 => self.exec_shl::<u64, u128>(bin, mode)?,
                    I64 => self.exec_shl::<i64, i128>(bin, mode)?,
                    Uw => self.exec_shl::<usize, usize>(bin, mode)?,
                    Iw => self.exec_shl::<isize, isize>(bin, mode)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                }

                Ok(ExecutionSuccess::Ok)
            }
            Shr(bin, ot, mode) => {
                match ot {
                    U8 => self.exec_shr::<u8, u16>(bin, mode)?,
                    I8 => self.exec_shr::<i8, i16>(bin, mode)?,
                    U16 => self.exec_shr::<u16, u32>(bin, mode)?,
                    I16 => self.exec_shr::<i16, i32>(bin, mode)?,
                    U32 => self.exec_shr::<u32, u64>(bin, mode)?,
                    I32 => self.exec_shr::<i32, i64>(bin, mode)?,
                    U64 => self.exec_shr::<u64, u128>(bin, mode)?,
                    I64 => self.exec_shr::<i64, i128>(bin, mode)?,
                    Uw => self.exec_shr::<usize, usize>(bin, mode)?,
                    Iw => self.exec_shr::<isize, isize>(bin, mode)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                }

                Ok(ExecutionSuccess::Ok)
            }
            And(bin, ot) => {
                match ot {
                    U8 => self.exec_and::<u8>(bin)?,
                    I8 => self.exec_and::<i8>(bin)?,
                    U16 => self.exec_and::<u16>(bin)?,
                    I16 => self.exec_and::<i16>(bin)?,
                    U32 => self.exec_and::<u32>(bin)?,
                    I32 => self.exec_and::<i32>(bin)?,
                    U64 => self.exec_and::<u64>(bin)?,
                    I64 => self.exec_and::<i64>(bin)?,
                    Uw => self.exec_and::<usize>(bin)?,
                    Iw => self.exec_and::<isize>(bin)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                }

                Ok(ExecutionSuccess::Ok)
            }
            Or(bin, ot) => {
                match ot {
                    U8 => self.exec_or::<u8>(bin)?,
                    I8 => self.exec_or::<i8>(bin)?,
                    U16 => self.exec_or::<u16>(bin)?,
                    I16 => self.exec_or::<i16>(bin)?,
                    U32 => self.exec_or::<u32>(bin)?,
                    I32 => self.exec_or::<i32>(bin)?,
                    U64 => self.exec_or::<u64>(bin)?,
                    I64 => self.exec_or::<i64>(bin)?,
                    Uw => self.exec_or::<usize>(bin)?,
                    Iw => self.exec_or::<isize>(bin)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                }

                Ok(ExecutionSuccess::Ok)
            }
            Xor(bin, ot) => {
                match ot {
                    U8 => self.exec_xor::<u8>(bin)?,
                    I8 => self.exec_xor::<i8>(bin)?,
                    U16 => self.exec_xor::<u16>(bin)?,
                    I16 => self.exec_xor::<i16>(bin)?,
                    U32 => self.exec_xor::<u32>(bin)?,
                    I32 => self.exec_xor::<i32>(bin)?,
                    U64 => self.exec_xor::<u64>(bin)?,
                    I64 => self.exec_xor::<i64>(bin)?,
                    Uw => self.exec_xor::<usize>(bin)?,
                    Iw => self.exec_xor::<isize>(bin)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                }

                Ok(ExecutionSuccess::Ok)
            }
            Not(un, ot) => {
                match ot {
                    U8 => self.exec_not::<u8>(un)?,
                    I8 => self.exec_not::<i8>(un)?,
                    U16 => self.exec_not::<u16>(un)?,
                    I16 => self.exec_not::<i16>(un)?,
                    U32 => self.exec_not::<u32>(un)?,
                    I32 => self.exec_not::<i32>(un)?,
                    U64 => self.exec_not::<u64>(un)?,
                    I64 => self.exec_not::<i64>(un)?,
                    Uw => self.exec_not::<usize>(un)?,
                    Iw => self.exec_not::<isize>(un)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                }

                Ok(ExecutionSuccess::Ok)
            }
            Neg(un, ot, mode) => {
                match ot {
                    U8 => self.exec_neg::<u8, u16>(un, mode)?,
                    I8 => self.exec_neg::<i8, i16>(un, mode)?,
                    U16 => self.exec_neg::<u16, u32>(un, mode)?,
                    I16 => self.exec_neg::<i16, i32>(un, mode)?,
                    U32 => self.exec_neg::<u32, u64>(un, mode)?,
                    I32 => self.exec_neg::<i32, i64>(un, mode)?,
                    U64 => self.exec_neg::<u64, u128>(un, mode)?,
                    I64 => self.exec_neg::<i64, i128>(un, mode)?,
                    Uw => self.exec_neg::<usize, usize>(un, mode)?,
                    Iw => self.exec_neg::<isize, isize>(un, mode)?,
                    F32 => self.exec_neg::<f32, f32>(un, mode)?,
                    F64 => self.exec_neg::<f64, f64>(un, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Inc(un, ot, mode) => {
                match ot {
                    U8 => self.exec_inc::<u8, u16>(un, mode)?,
                    I8 => self.exec_inc::<i8, i16>(un, mode)?,
                    U16 => self.exec_inc::<u16, u32>(un, mode)?,
                    I16 => self.exec_inc::<i16, i32>(un, mode)?,
                    U32 => self.exec_inc::<u32, u64>(un, mode)?,
                    I32 => self.exec_inc::<i32, i64>(un, mode)?,
                    U64 => self.exec_inc::<u64, u128>(un, mode)?,
                    I64 => self.exec_inc::<i64, i128>(un, mode)?,
                    Uw => self.exec_inc::<usize, usize>(un, mode)?,
                    Iw => self.exec_inc::<isize, isize>(un, mode)?,
                    F32 => self.exec_inc::<f32, f32>(un, mode)?,
                    F64 => self.exec_inc::<f64, f64>(un, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Dec(un, ot, mode) => {
                match ot {
                    U8 => self.exec_dec::<u8, u16>(un, mode)?,
                    I8 => self.exec_dec::<i8, i16>(un, mode)?,
                    U16 => self.exec_dec::<u16, u32>(un, mode)?,
                    I16 => self.exec_dec::<i16, i32>(un, mode)?,
                    U32 => self.exec_dec::<u32, u64>(un, mode)?,
                    I32 => self.exec_dec::<i32, i64>(un, mode)?,
                    U64 => self.exec_dec::<u64, u128>(un, mode)?,
                    I64 => self.exec_dec::<i64, i128>(un, mode)?,
                    Uw => self.exec_dec::<usize, usize>(un, mode)?,
                    Iw => self.exec_dec::<isize, isize>(un, mode)?,
                    F32 => self.exec_dec::<f32, f32>(un, mode)?,
                    F64 => self.exec_dec::<f64, f64>(un, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Go(x) => {
                self.program_counter = self.get_val(x)?;
                return Ok(ExecutionSuccess::Ok);
            }
            Ift(un, ot) => {
                let res = match ot {
                    U8 => self.get_un::<u8>(un)? != 0,
                    I8 => self.get_un::<i8>(un)? != 0,
                    U16 => self.get_un::<u16>(un)? != 0,
                    I16 => self.get_un::<i16>(un)? != 0,
                    U32 => self.get_un::<u32>(un)? != 0,
                    I32 => self.get_un::<i32>(un)? != 0,
                    U64 => self.get_un::<u64>(un)? != 0,
                    I64 => self.get_un::<i64>(un)? != 0,
                    Uw => self.get_un::<usize>(un)? != 0,
                    Iw => self.get_un::<isize>(un)? != 0,
                    F32 => self.get_un::<f32>(un)? != 0.0,
                    F64 => self.get_un::<f64>(un)? != 0.0,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Iff(un, ot) => {
                let res = match ot {
                    U8 => self.get_un::<u8>(un)? == 0,
                    I8 => self.get_un::<i8>(un)? == 0,
                    U16 => self.get_un::<u16>(un)? == 0,
                    I16 => self.get_un::<i16>(un)? == 0,
                    U32 => self.get_un::<u32>(un)? == 0,
                    I32 => self.get_un::<i32>(un)? == 0,
                    U64 => self.get_un::<u64>(un)? == 0,
                    I64 => self.get_un::<i64>(un)? == 0,
                    Uw => self.get_un::<usize>(un)? == 0,
                    Iw => self.get_un::<isize>(un)? == 0,
                    F32 => self.get_un::<f32>(un)? == 0.0,
                    F64 => self.get_un::<f64>(un)? == 0.0,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ife(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ife::<u8>(bo)?,
                    I8 => self.exec_ife::<i8>(bo)?,
                    U16 => self.exec_ife::<u16>(bo)?,
                    I16 => self.exec_ife::<i16>(bo)?,
                    U32 => self.exec_ife::<u32>(bo)?,
                    I32 => self.exec_ife::<i32>(bo)?,
                    U64 => self.exec_ife::<u64>(bo)?,
                    I64 => self.exec_ife::<i64>(bo)?,
                    Uw => self.exec_ife::<usize>(bo)?,
                    Iw => self.exec_ife::<isize>(bo)?,
                    F32 => self.exec_ife::<f32>(bo)?,
                    F64 => self.exec_ife::<f64>(bo)?,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ifl(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ifl::<u8>(bo)?,
                    I8 => self.exec_ifl::<i8>(bo)?,
                    U16 => self.exec_ifl::<u16>(bo)?,
                    I16 => self.exec_ifl::<i16>(bo)?,
                    U32 => self.exec_ifl::<u32>(bo)?,
                    I32 => self.exec_ifl::<i32>(bo)?,
                    U64 => self.exec_ifl::<u64>(bo)?,
                    I64 => self.exec_ifl::<i64>(bo)?,
                    Uw => self.exec_ifl::<usize>(bo)?,
                    Iw => self.exec_ifl::<isize>(bo)?,
                    F32 => self.exec_ifl::<f32>(bo)?,
                    F64 => self.exec_ifl::<f64>(bo)?,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ifg(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ifg::<u8>(bo)?,
                    I8 => self.exec_ifg::<i8>(bo)?,
                    U16 => self.exec_ifg::<u16>(bo)?,
                    I16 => self.exec_ifg::<i16>(bo)?,
                    U32 => self.exec_ifg::<u32>(bo)?,
                    I32 => self.exec_ifg::<i32>(bo)?,
                    U64 => self.exec_ifg::<u64>(bo)?,
                    I64 => self.exec_ifg::<i64>(bo)?,
                    Uw => self.exec_ifg::<usize>(bo)?,
                    Iw => self.exec_ifg::<isize>(bo)?,
                    F32 => self.exec_ifg::<f32>(bo)?,
                    F64 => self.exec_ifg::<f64>(bo)?,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ine(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ine::<u8>(bo)?,
                    I8 => self.exec_ine::<i8>(bo)?,
                    U16 => self.exec_ine::<u16>(bo)?,
                    I16 => self.exec_ine::<i16>(bo)?,
                    U32 => self.exec_ine::<u32>(bo)?,
                    I32 => self.exec_ine::<i32>(bo)?,
                    U64 => self.exec_ine::<u64>(bo)?,
                    I64 => self.exec_ine::<i64>(bo)?,
                    Uw => self.exec_ine::<usize>(bo)?,
                    Iw => self.exec_ine::<isize>(bo)?,
                    F32 => self.exec_ine::<f32>(bo)?,
                    F64 => self.exec_ine::<f64>(bo)?,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Inl(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_inl::<u8>(bo)?,
                    I8 => self.exec_inl::<i8>(bo)?,
                    U16 => self.exec_inl::<u16>(bo)?,
                    I16 => self.exec_inl::<i16>(bo)?,
                    U32 => self.exec_inl::<u32>(bo)?,
                    I32 => self.exec_inl::<i32>(bo)?,
                    U64 => self.exec_inl::<u64>(bo)?,
                    I64 => self.exec_inl::<i64>(bo)?,
                    Uw => self.exec_inl::<usize>(bo)?,
                    Iw => self.exec_inl::<isize>(bo)?,
                    F32 => self.exec_inl::<f32>(bo)?,
                    F64 => self.exec_inl::<f64>(bo)?,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ing(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ing::<u8>(bo)?,
                    I8 => self.exec_ing::<i8>(bo)?,
                    U16 => self.exec_ing::<u16>(bo)?,
                    I16 => self.exec_ing::<i16>(bo)?,
                    U32 => self.exec_ing::<u32>(bo)?,
                    I32 => self.exec_ing::<i32>(bo)?,
                    U64 => self.exec_ing::<u64>(bo)?,
                    I64 => self.exec_ing::<i64>(bo)?,
                    Uw => self.exec_ing::<usize>(bo)?,
                    Iw => self.exec_ing::<isize>(bo)?,
                    F32 => self.exec_ing::<f32>(bo)?,
                    F64 => self.exec_ing::<f64>(bo)?,
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ifa(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ifa::<u8>(bo)?,
                    I8 => self.exec_ifa::<i8>(bo)?,
                    U16 => self.exec_ifa::<u16>(bo)?,
                    I16 => self.exec_ifa::<i16>(bo)?,
                    U32 => self.exec_ifa::<u32>(bo)?,
                    I32 => self.exec_ifa::<i32>(bo)?,
                    U64 => self.exec_ifa::<u64>(bo)?,
                    I64 => self.exec_ifa::<i64>(bo)?,
                    Uw => self.exec_ifa::<usize>(bo)?,
                    Iw => self.exec_ifa::<isize>(bo)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ifo(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ifo::<u8>(bo)?,
                    I8 => self.exec_ifo::<i8>(bo)?,
                    U16 => self.exec_ifo::<u16>(bo)?,
                    I16 => self.exec_ifo::<i16>(bo)?,
                    U32 => self.exec_ifo::<u32>(bo)?,
                    I32 => self.exec_ifo::<i32>(bo)?,
                    U64 => self.exec_ifo::<u64>(bo)?,
                    I64 => self.exec_ifo::<i64>(bo)?,
                    Uw => self.exec_ifo::<usize>(bo)?,
                    Iw => self.exec_ifo::<isize>(bo)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ifx(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ifx::<u8>(bo)?,
                    I8 => self.exec_ifx::<i8>(bo)?,
                    U16 => self.exec_ifx::<u16>(bo)?,
                    I16 => self.exec_ifx::<i16>(bo)?,
                    U32 => self.exec_ifx::<u32>(bo)?,
                    I32 => self.exec_ifx::<i32>(bo)?,
                    U64 => self.exec_ifx::<u64>(bo)?,
                    I64 => self.exec_ifx::<i64>(bo)?,
                    Uw => self.exec_ifx::<usize>(bo)?,
                    Iw => self.exec_ifx::<isize>(bo)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ina(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ina::<u8>(bo)?,
                    I8 => self.exec_ina::<i8>(bo)?,
                    U16 => self.exec_ina::<u16>(bo)?,
                    I16 => self.exec_ina::<i16>(bo)?,
                    U32 => self.exec_ina::<u32>(bo)?,
                    I32 => self.exec_ina::<i32>(bo)?,
                    U64 => self.exec_ina::<u64>(bo)?,
                    I64 => self.exec_ina::<i64>(bo)?,
                    Uw => self.exec_ina::<usize>(bo)?,
                    Iw => self.exec_ina::<isize>(bo)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Ino(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_ino::<u8>(bo)?,
                    I8 => self.exec_ino::<i8>(bo)?,
                    U16 => self.exec_ino::<u16>(bo)?,
                    I16 => self.exec_ino::<i16>(bo)?,
                    U32 => self.exec_ino::<u32>(bo)?,
                    I32 => self.exec_ino::<i32>(bo)?,
                    U64 => self.exec_ino::<u64>(bo)?,
                    I64 => self.exec_ino::<i64>(bo)?,
                    Uw => self.exec_ino::<usize>(bo)?,
                    Iw => self.exec_ino::<isize>(bo)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            Inx(bo, ot) => {
                let res = match ot {
                    U8 => self.exec_inx::<u8>(bo)?,
                    I8 => self.exec_inx::<i8>(bo)?,
                    U16 => self.exec_inx::<u16>(bo)?,
                    I16 => self.exec_inx::<i16>(bo)?,
                    U32 => self.exec_inx::<u32>(bo)?,
                    I32 => self.exec_inx::<i32>(bo)?,
                    U64 => self.exec_inx::<u64>(bo)?,
                    I64 => self.exec_inx::<i64>(bo)?,
                    Uw => self.exec_inx::<usize>(bo)?,
                    Iw => self.exec_inx::<isize>(bo)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                };

                if res {
                    Ok(ExecutionSuccess::Ok)
                } else {
                    self.pass_condition()?;
                    return Ok(ExecutionSuccess::Ok);
                }
            }
            App(x) => {
                self.app(self.get_val(x)?)?;
                Ok(ExecutionSuccess::Ok)
            }
            Par(un, ot, mode) => {
                match ot {
                    U8 => self.exec_par::<u8>(un, mode)?,
                    I8 => self.exec_par::<i8>(un, mode)?,
                    U16 => self.exec_par::<u16>(un, mode)?,
                    I16 => self.exec_par::<i16>(un, mode)?,
                    U32 => self.exec_par::<u32>(un, mode)?,
                    I32 => self.exec_par::<i32>(un, mode)?,
                    U64 => self.exec_par::<u64>(un, mode)?,
                    I64 => self.exec_par::<i64>(un, mode)?,
                    Uw => self.exec_par::<usize>(un, mode)?,
                    Iw => self.exec_par::<isize>(un, mode)?,
                    F32 => self.exec_par::<f32>(un, mode)?,
                    F64 => self.exec_par::<f64>(un, mode)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Clf(x) => {
                self.clf(self.get_val(x)?)?;
                return Ok(ExecutionSuccess::Ok);
            }
            Ret => {
                self.ret()?;
                return Ok(ExecutionSuccess::Ok);
            }
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
                program: &[
                    Op::Nop
                ],
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

        assert_eq!(
            exe.set_val(Operand::Val(7), 0),
            Err(ExecutionError::IncorrectOperation(Op::Nop)),
        );
        assert_eq!(exe.get_val::<usize>(Operand::Val(8)), Ok(8));

        assert_eq!(
            exe.set_val(Operand::Ref(0), 0),
            Err(ExecutionError::IncorrectOperation(Op::Nop)),
        );
        assert_eq!(exe.get_val::<usize>(Operand::Ref(0)), Ok(8));

        assert_eq!(
            exe.set_val(Operand::Emp, 0),
            Err(ExecutionError::IncorrectOperation(Op::Nop)),
        );
        assert_eq!(
            exe.get_val::<usize>(Operand::Emp),
            Err(ExecutionError::IncorrectOperation(Op::Nop)),
        );
    }

    #[test]
    fn executor_set() {
        let fb = f32::to_le_bytes(0.123);
        let float = usize::from_le_bytes([fb[0], fb[1], fb[2], fb[3], 0, 0, 0, 0]);

        let functions = [
            Function {
                frame_size: 4,
                program: &[
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

        assert_eq!(exe.execute(), Executed::Err(ExecutionError::IncorrectOperation(
            Op::Set(BinOp::new(Operand::Val(0), Operand::Val(12)), OpType::I32)
        )));
        exe.program_counter += 1; // Move manually after incorrect operation

        assert_eq!(exe.execute(), Executed::Err(ExecutionError::IncorrectOperation(
            Op::Set(BinOp::new(Operand::Emp, Operand::Val(12)), OpType::I32)
        )));
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
                program: &[
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

    #[test]
    fn executor_mul() {
        let functions = [
            Function {
                frame_size: 8,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(8)), OpType::I32),
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Val(5)), OpType::I32),
                    Op::Mul(
                        BinOp::new(Operand::Loc(0), Operand::Val(2)),
                        OpType::I32,
                        ArithmeticMode::default(),
                    ),
                    Op::Mul(
                        BinOp::new(Operand::Loc(4), Operand::Val(2)),
                        OpType::I32,
                        ArithmeticMode::default(),
                    ),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(0)), Ok(16));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(4)), Ok(10));
    }

    #[test]
    fn executor_div() {
        let functions = [
            Function {
                frame_size: 8,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(8)), OpType::I32),
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Val(5)), OpType::I32),
                    Op::Div(BinOp::new(Operand::Loc(0), Operand::Val(2)), OpType::I32),
                    Op::Div(BinOp::new(Operand::Loc(4), Operand::Val(2)), OpType::I32),
                    Op::Div(BinOp::new(Operand::Loc(0), Operand::Val(0)), OpType::I32),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(0)), Ok(4));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<i32>(Operand::Loc(4)), Ok(2));
        assert_eq!(exe.execute(), Executed::Err(ExecutionError::DivisionByZero));
    }

    #[test]
    fn executor_go() {
        let functions = [
            Function {
                frame_size: 4,
                program: &[
                    Op::Inc(UnOp::new(Operand::Loc(0)), OpType::U32, ArithmeticMode::default()),
                    Op::Go(Operand::Val(0)),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(1));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(2));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(3));
    }

    #[test]
    fn executor_ift() {
        let functions = [
            Function {
                frame_size: 1,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(1)), OpType::U8),
                    Op::Ift(UnOp::new(Operand::Loc(0)), OpType::U8),
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(2)), OpType::U8),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u8>(Operand::Loc(0)), Ok(1));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u8>(Operand::Loc(0)), Ok(2));
    }

    #[test]
    fn executor_iff() {
        let functions = [
            Function {
                frame_size: 1,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(1)), OpType::U8),
                    Op::Iff(UnOp::new(Operand::Loc(0)), OpType::U8),
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(2)), OpType::U8),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u8>(Operand::Loc(0)), Ok(1));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u8>(Operand::Loc(0)), Ok(1));
        assert_eq!(exe.execute(), Executed::Err(ExecutionError::EndOfProgram));
    }

    #[test]
    fn executor_ife() {
        let functions = [
            Function {
                frame_size: 8,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(32)), OpType::U32),
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Val(32)), OpType::U32),
                    Op::Ife(BinOp::new(Operand::Loc(0), Operand::Loc(4)), OpType::U32),
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(1)), OpType::U32),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(32));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(4)), Ok(32));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(1));
    }

    #[test]
    fn executor_ifa() {
        let functions = [
            Function {
                frame_size: 8,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(32)), OpType::U32),
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Val(2)), OpType::U32),
                    Op::Ifa(BinOp::new(Operand::Loc(0), Operand::Loc(4)), OpType::U32),
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(1)), OpType::U32),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(32));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(4)), Ok(2));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Err(ExecutionError::EndOfProgram));
    }

    #[test]
    fn executor_ina() {
        let functions = [
            Function {
                frame_size: 8,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(32)), OpType::U32),
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Val(2)), OpType::U32),
                    Op::Ina(BinOp::new(Operand::Loc(0), Operand::Loc(4)), OpType::U32),
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Val(1)), OpType::U32),
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(32));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(4)), Ok(2));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(1));
    }

    #[test]
    fn executor_call_fn() {
        let functions = [
            Function {
                frame_size: 4,
                program: &[
                    Op::App(Operand::Val(1)),
                    Op::Par(
                        UnOp::new(Operand::Val(2)),
                        OpType::I32,
                        ParameterMode::default(),
                    ),
                    Op::Clf(Operand::Val(0)),
                    Op::Ret,
                ],
            },
            Function {
                frame_size: 8,
                program: &[
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Val(3)), OpType::I32),
                    Op::Add(
                        BinOp::new(Operand::Ret(0), Operand::Loc(0)),
                        OpType::I32,
                        ArithmeticMode::default(),
                    ),
                    Op::Add(
                        BinOp::new(Operand::Ret(0), Operand::Loc(4)),
                        OpType::I32,
                        ArithmeticMode::default(),
                    ),
                    Op::Ret,
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.call_stack.len(), 2);

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert_eq!(exe.call_stack.len(), 1);

        assert_eq!(exe.get_val::<i32>(Operand::Loc(0)), Ok(5));

        assert_eq!(exe.execute(), Executed::Ok(ExecutionSuccess::Ok));
        assert!(exe.call_stack.is_empty());
    }

    #[test]
    fn executor_gcd() {
        let functions = [
            Function {
                frame_size: 12,
                program: &[
                    // u32 result
                    // u32 x
                    // u32 y
                    // set x 234
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Val(234)), OpType::U32),
                    // set y 533
                    Op::Set(BinOp::new(Operand::Loc(8), Operand::Val(533)), OpType::U32),
                    // app gcd
                    Op::App(Operand::Val(1)),
                    // par x
                    Op::Par(
                        UnOp::new(Operand::Loc(4)),
                        OpType::U32,
                        ParameterMode::default(),
                    ),
                    // par y
                    Op::Par(
                        UnOp::new(Operand::Loc(8)),
                        OpType::U32,
                        ParameterMode::default(),
                    ),
                    // clf result
                    Op::Clf(Operand::Val(0)),
                    // end
                    Op::End(Operand::Val(0)),
                ],
            },
            Function {
                // fn gcd
                frame_size: 12,
                program: &[
                    // u32 a
                    // u32 b
                    // u32 c
                    // loop:
                    // set c a
                    Op::Set(BinOp::new(Operand::Loc(8), Operand::Loc(0)), OpType::U32),
                    // mod c b
                    Op::Mod(BinOp::new(Operand::Loc(8), Operand::Loc(4)), OpType::U32),
                    // set a b
                    Op::Set(BinOp::new(Operand::Loc(0), Operand::Loc(4)), OpType::U32),
                    // set b c
                    Op::Set(BinOp::new(Operand::Loc(4), Operand::Loc(8)), OpType::U32),
                    // ift b
                    Op::Ift(UnOp::new(Operand::Loc(4)), OpType::U32),
                    // go loop
                    Op::Go(Operand::Val(0)),
                    // set ^ a
                    Op::Set(BinOp::new(Operand::Ret(0), Operand::Loc(0)), OpType::U32),
                    // ret
                    Op::Ret,
                ],
            },
        ];

        let mut exe = Executor::new(&functions);
        exe.call(0, 0).unwrap();

        let mut executed = Executed::Ok(ExecutionSuccess::Ok);
        while let Executed::Ok(ExecutionSuccess::Ok) = executed {
            executed = exe.execute();
        };

        assert_eq!(executed, Executed::Ok(ExecutionSuccess::End(0)));
        assert_eq!(exe.get_val::<u32>(Operand::Loc(0)), Ok(13));
    }
}
