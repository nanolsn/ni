#[cfg(test)]
mod tests;

use common::*;
use super::{
    memory::*,
    primary::*,
};

#[derive(Debug)]
pub struct Function<'f> {
    frame_size: UWord,
    program: &'f [Op],
}

#[derive(Debug)]
pub struct FunctionCall<'f> {
    function: &'f Function<'f>,
    base_address: UWord,
    ret_address: UWord,
    ret_program_counter: UWord,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionError {
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
    End(UWord),
    Sleep(UWord),
}

pub type Executed = Result<ExecutionSuccess, ExecutionError>;

#[derive(Debug)]
pub struct Executor<'f> {
    functions: &'f [Function<'f>],
    memory: Memory,
    program_counter: UWord,
    call_stack: Vec<FunctionCall<'f>>,
    prepared_call: bool,
    parameter_ptr: UWord,
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

macro_rules! impl_cnv {
    ($t:ty, $obj:ident, $uid:ident, $x:ident, $y:ident) => {
        match $uid {
            U8 => $obj.exec_cnv::<$t, u8>($x, $y)?,
            I8 => $obj.exec_cnv::<$t, i8>($x, $y)?,
            U16 => $obj.exec_cnv::<$t, u16>($x, $y)?,
            I16 => $obj.exec_cnv::<$t, i16>($x, $y)?,
            U32 => $obj.exec_cnv::<$t, u32>($x, $y)?,
            I32 => $obj.exec_cnv::<$t, i32>($x, $y)?,
            U64 => $obj.exec_cnv::<$t, u64>($x, $y)?,
            I64 => $obj.exec_cnv::<$t, i64>($x, $y)?,
            Uw => $obj.exec_cnv::<$t, UWord>($x, $y)?,
            Iw => $obj.exec_cnv::<$t, IWord>($x, $y)?,
            F32 => $obj.exec_cnv::<$t, f32>($x, $y)?,
            F64 => $obj.exec_cnv::<$t, f64>($x, $y)?,
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

    fn app(&mut self, function_id: UWord) -> Result<(), ExecutionError> {
        let f = self.functions
            .get(function_id as usize)
            .ok_or(ExecutionError::UnknownFunction)?;

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

    fn clf(&mut self, ret_address: UWord) -> Result<(), ExecutionError> {
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

    pub fn call(&mut self, function_id: UWord, ret_address: UWord) -> Result<(), ExecutionError> {
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
            .get(self.program_counter as usize)
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
            Operand::Val(val) => T::from_word(val),
            Operand::Ref(var) => T::from_word(
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
        let a_offset: UWord = self.get_val(offset)?;
        Ok(a.map(|a| a.wrapping_add(a_offset)))
    }

    fn exec_set<T>(&mut self, bin: BinOp) -> Result<(), ExecutionError>
        where
            T: Primary,
    { self.update_bin::<T, T, _>(bin, |_, y| y) }

    fn exec_cnv<T, U>(&mut self, left: Operand, right: Operand) -> Result<(), ExecutionError>
        where
            T: Primary,
            U: Convert<T>,
    { self.set_val(left, U::convert(self.get_val(right)?)) }

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

    fn exec_shl<T>(&mut self, x: Operand, y: Operand) -> Result<(), ExecutionError>
        where
            T: Shl
    {
        let x_val: T = self.get_val(x)?;
        let y_val: u8 = self.get_val(y)?;
        self.set_val(x, x_val.wrapping(y_val))
    }

    fn exec_shr<T>(&mut self, x: Operand, y: Operand) -> Result<(), ExecutionError>
        where
            T: Shr
    {
        let x_val: T = self.get_val(x)?;
        let y_val: u8 = self.get_val(y)?;
        self.set_val(x, x_val.wrapping(y_val))
    }

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
        self.parameter_ptr = self.parameter_ptr.wrapping_add(T::SIZE as UWord);

        match mode {
            ParameterMode::Set => {
                let val = self.get_un(un)?;
                self.set_val::<T>(Operand::Loc(parameter_loc), val)?;
            }
            ParameterMode::Emp => {}
            ParameterMode::Zer => self.set_val::<T>(
                Operand::Loc(parameter_loc),
                T::zero(),
            )?,
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
                    Uw => self.exec_set::<UWord>(bin)?,
                    Iw => self.exec_set::<IWord>(bin)?,
                    F32 => self.exec_set::<f32>(bin)?,
                    F64 => self.exec_set::<f64>(bin)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Cnv(x, y, t, u) => {
                match t {
                    U8 => impl_cnv!(u8, self, u, x, y),
                    I8 => impl_cnv!(i8, self, u, x, y),
                    U16 => impl_cnv!(u16, self, u, x, y),
                    I16 => impl_cnv!(i16, self, u, x, y),
                    U32 => impl_cnv!(u32, self, u, x, y),
                    I32 => impl_cnv!(i32, self, u, x, y),
                    U64 => impl_cnv!(u64, self, u, x, y),
                    I64 => impl_cnv!(i64, self, u, x, y),
                    Uw => impl_cnv!(UWord, self, u, x, y),
                    Iw => impl_cnv!(IWord, self, u, x, y),
                    F32 => impl_cnv!(f32, self, u, x, y),
                    F64 => impl_cnv!(f64, self, u, x, y),
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
                    Uw => self.exec_add::<UWord, UWord>(bin, mode)?,
                    Iw => self.exec_add::<IWord, IWord>(bin, mode)?,
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
                    Uw => self.exec_sub::<UWord, UWord>(bin, mode)?,
                    Iw => self.exec_sub::<IWord, IWord>(bin, mode)?,
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
                    Uw => self.exec_mul::<UWord, UWord>(bin, mode)?,
                    Iw => self.exec_mul::<IWord, IWord>(bin, mode)?,
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
                    Uw => self.exec_div::<UWord>(bin)?,
                    Iw => self.exec_div::<IWord>(bin)?,
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
                    Uw => self.exec_mod::<UWord>(bin)?,
                    Iw => self.exec_mod::<IWord>(bin)?,
                    F32 => self.exec_mod::<f32>(bin)?,
                    F64 => self.exec_mod::<f64>(bin)?,
                }

                Ok(ExecutionSuccess::Ok)
            }
            Shl(x, y, ot) => {
                match ot {
                    U8 => self.exec_shl::<u8>(x, y)?,
                    I8 => self.exec_shl::<i8>(x, y)?,
                    U16 => self.exec_shl::<u16>(x, y)?,
                    I16 => self.exec_shl::<i16>(x, y)?,
                    U32 => self.exec_shl::<u32>(x, y)?,
                    I32 => self.exec_shl::<i32>(x, y)?,
                    U64 => self.exec_shl::<u64>(x, y)?,
                    I64 => self.exec_shl::<i64>(x, y)?,
                    Uw => self.exec_shl::<UWord>(x, y)?,
                    Iw => self.exec_shl::<IWord>(x, y)?,
                    F32 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                    F64 => return Err(ExecutionError::IncorrectOperation(*self.current_op()?)),
                }

                Ok(ExecutionSuccess::Ok)
            }
            Shr(x, y, ot) => {
                match ot {
                    U8 => self.exec_shr::<u8>(x, y)?,
                    I8 => self.exec_shr::<i8>(x, y)?,
                    U16 => self.exec_shr::<u16>(x, y)?,
                    I16 => self.exec_shr::<i16>(x, y)?,
                    U32 => self.exec_shr::<u32>(x, y)?,
                    I32 => self.exec_shr::<i32>(x, y)?,
                    U64 => self.exec_shr::<u64>(x, y)?,
                    I64 => self.exec_shr::<i64>(x, y)?,
                    Uw => self.exec_shr::<UWord>(x, y)?,
                    Iw => self.exec_shr::<IWord>(x, y)?,
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
                    Uw => self.exec_and::<UWord>(bin)?,
                    Iw => self.exec_and::<IWord>(bin)?,
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
                    Uw => self.exec_or::<UWord>(bin)?,
                    Iw => self.exec_or::<IWord>(bin)?,
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
                    Uw => self.exec_xor::<UWord>(bin)?,
                    Iw => self.exec_xor::<IWord>(bin)?,
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
                    Uw => self.exec_not::<UWord>(un)?,
                    Iw => self.exec_not::<IWord>(un)?,
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
                    Uw => self.exec_neg::<UWord, UWord>(un, mode)?,
                    Iw => self.exec_neg::<IWord, IWord>(un, mode)?,
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
                    Uw => self.exec_inc::<UWord, UWord>(un, mode)?,
                    Iw => self.exec_inc::<IWord, IWord>(un, mode)?,
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
                    Uw => self.exec_dec::<UWord, UWord>(un, mode)?,
                    Iw => self.exec_dec::<IWord, IWord>(un, mode)?,
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
                    Uw => self.get_un::<UWord>(un)? != 0,
                    Iw => self.get_un::<IWord>(un)? != 0,
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
                    Uw => self.get_un::<UWord>(un)? == 0,
                    Iw => self.get_un::<IWord>(un)? == 0,
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
                    Uw => self.exec_ife::<UWord>(bo)?,
                    Iw => self.exec_ife::<IWord>(bo)?,
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
                    Uw => self.exec_ifl::<UWord>(bo)?,
                    Iw => self.exec_ifl::<IWord>(bo)?,
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
                    Uw => self.exec_ifg::<UWord>(bo)?,
                    Iw => self.exec_ifg::<IWord>(bo)?,
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
                    Uw => self.exec_ine::<UWord>(bo)?,
                    Iw => self.exec_ine::<IWord>(bo)?,
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
                    Uw => self.exec_inl::<UWord>(bo)?,
                    Iw => self.exec_inl::<IWord>(bo)?,
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
                    Uw => self.exec_ing::<UWord>(bo)?,
                    Iw => self.exec_ing::<IWord>(bo)?,
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
                    Uw => self.exec_ifa::<UWord>(bo)?,
                    Iw => self.exec_ifa::<IWord>(bo)?,
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
                    Uw => self.exec_ifo::<UWord>(bo)?,
                    Iw => self.exec_ifo::<IWord>(bo)?,
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
                    Uw => self.exec_ifx::<UWord>(bo)?,
                    Iw => self.exec_ifx::<IWord>(bo)?,
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
                    Uw => self.exec_ina::<UWord>(bo)?,
                    Iw => self.exec_ina::<IWord>(bo)?,
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
                    Uw => self.exec_ino::<UWord>(bo)?,
                    Iw => self.exec_ino::<IWord>(bo)?,
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
                    Uw => self.exec_inx::<UWord>(bo)?,
                    Iw => self.exec_inx::<IWord>(bo)?,
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
                    Uw => self.exec_par::<UWord>(un, mode)?,
                    Iw => self.exec_par::<IWord>(un, mode)?,
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
