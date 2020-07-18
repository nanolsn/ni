use common::UWord;

pub trait Primary: Sized {
    const SIZE: usize = std::mem::size_of::<Self>();

    type Bytes: std::borrow::Borrow<[u8]>;

    fn to_bytes(&self) -> Self::Bytes;

    fn from_bytes(bytes: Self::Bytes) -> Self;

    fn from_slice(slice: &[u8]) -> Self;

    fn zero() -> Self;

    fn one() -> Self;

    fn from_word(val: UWord) -> Self;
}

macro_rules! impl_primary {
    ($($t:ty),+) => {
        $(
        impl Primary for $t {
            type Bytes = [u8; std::mem::size_of::<Self>()];

            fn to_bytes(&self) -> Self::Bytes { self.to_le_bytes() }

            fn from_bytes(bytes: Self::Bytes) -> Self { Self::from_le_bytes(bytes) }

            fn from_slice(slice: &[u8]) -> Self {
                let mut buf = [0; Self::SIZE];

                for (i, b) in slice.iter().enumerate() {
                    buf[i] = *b;
                }

                Self::from_bytes(buf)
            }

            fn zero() -> Self { 0 as $t }

            fn one() -> Self { 1 as $t }

            fn from_word(val: UWord) -> Self {
                let mut buf = [0; Self::SIZE];
                let ubytes = UWord::to_le_bytes(val);

                for i in 0..Self::SIZE.min(std::mem::size_of::<UWord>()) {
                    buf[i] = ubytes[i];
                }

                Self::from_bytes(buf)
            }
        }
        )+
    }
}

impl_primary!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64);

pub trait Add: Primary {
    fn wrapping(self, r: Self) -> Self;
    fn saturating(self, r: Self) -> Self;
    fn checked(self, r: Self) -> Option<Self>;
}

macro_rules! impl_add {
    ($($t:ty),+) => {
        $(
        impl Add for $t {
            fn wrapping(self, r: Self) -> Self { self.wrapping_add(r) }
            fn saturating(self, r: Self) -> Self { self.saturating_add(r) }
            fn checked(self, r: Self) -> Option<Self> { self.checked_add(r) }
        }
        )+
    }
}

impl_add!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Add for f32 {
    fn wrapping(self, r: Self) -> Self { self + r }
    fn saturating(self, r: Self) -> Self { self + r }
    fn checked(self, r: Self) -> Option<Self> { Some(self + r) }
}

impl Add for f64 {
    fn wrapping(self, r: Self) -> Self { self + r }
    fn saturating(self, r: Self) -> Self { self + r }
    fn checked(self, r: Self) -> Option<Self> { Some(self + r) }
}

pub trait Sub: Primary {
    fn wrapping(self, r: Self) -> Self;
    fn saturating(self, r: Self) -> Self;
    fn checked(self, r: Self) -> Option<Self>;
}

macro_rules! impl_sub {
    ($($t:ty),+) => {
        $(
        impl Sub for $t {
            fn wrapping(self, r: Self) -> Self { self.wrapping_sub(r) }
            fn saturating(self, r: Self) -> Self { self.saturating_sub(r) }
            fn checked(self, r: Self) -> Option<Self> { self.checked_sub(r) }
        }
        )+
    }
}

impl_sub!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Sub for f32 {
    fn wrapping(self, r: Self) -> Self { self - r }
    fn saturating(self, r: Self) -> Self { self - r }
    fn checked(self, r: Self) -> Option<Self> { Some(self - r) }
}

impl Sub for f64 {
    fn wrapping(self, r: Self) -> Self { self - r }
    fn saturating(self, r: Self) -> Self { self - r }
    fn checked(self, r: Self) -> Option<Self> { Some(self - r) }
}

pub trait Mul: Primary {
    fn wrapping(self, r: Self) -> Self;
    fn saturating(self, r: Self) -> Self;
    fn checked(self, r: Self) -> Option<Self>;
}

macro_rules! impl_mul {
    ($($t:ty),+) => {
        $(
        impl Mul for $t {
            fn wrapping(self, r: Self) -> Self { self.wrapping_mul(r) }
            fn saturating(self, r: Self) -> Self { self.saturating_mul(r) }
            fn checked(self, r: Self) -> Option<Self> { self.checked_mul(r) }
        }
        )+
    }
}

impl_mul!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Mul for f32 {
    fn wrapping(self, r: Self) -> Self { self * r }
    fn saturating(self, r: Self) -> Self { self * r }
    fn checked(self, r: Self) -> Option<Self> { Some(self * r) }
}

impl Mul for f64 {
    fn wrapping(self, r: Self) -> Self { self * r }
    fn saturating(self, r: Self) -> Self { self * r }
    fn checked(self, r: Self) -> Option<Self> { Some(self * r) }
}

pub trait Div: Primary {
    fn wrapping(self, r: Self) -> Self;
}

macro_rules! impl_div {
    ($($t:ty),+) => {
        $(
        impl Div for $t {
            fn wrapping(self, r: Self) -> Self { self.wrapping_div(r) }
        }
        )+
    }
}

impl_div!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Div for f32 {
    fn wrapping(self, r: Self) -> Self { self / r }
}

impl Div for f64 {
    fn wrapping(self, r: Self) -> Self { self / r }
}

pub trait Rem: Primary {
    fn wrapping(self, r: Self) -> Self;
}

macro_rules! impl_rem {
    ($($t:ty),+) => {
        $(
        impl Rem for $t {
            fn wrapping(self, r: Self) -> Self { self.wrapping_rem(r) }
        }
        )+
    }
}

impl_rem!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Rem for f32 {
    fn wrapping(self, r: Self) -> Self { self % r }
}

impl Rem for f64 {
    fn wrapping(self, r: Self) -> Self { self % r }
}

pub trait Shl: Primary {
    fn wrapping(self, r: u8) -> Self;
    fn saturating(self, r: u8) -> Self;
    fn checked(self, r: u8) -> Option<Self>;
}

macro_rules! impl_shl {
    ($($t:ty),+) => {
        $(
        impl Shl for $t {
            fn wrapping(self, r: u8) -> Self { self.wrapping_shl(r as u32) }
            fn saturating(self, r: u8) -> Self { self.wrapping_shl(r as u32) }
            fn checked(self, r: u8) -> Option<Self> { self.checked_shl(r as u32) }
        }
        )+
    }
}

impl_shl!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

pub trait Shr: Primary {
    fn wrapping(self, r: u8) -> Self;
    fn saturating(self, r: u8) -> Self;
    fn checked(self, r: u8) -> Option<Self>;
}

macro_rules! impl_shr {
    ($($t:ty),+) => {
        $(
        impl Shr for $t {
            fn wrapping(self, r: u8) -> Self { self.wrapping_shr(r as u32) }
            fn saturating(self, r: u8) -> Self { self.wrapping_shr(r as u32) }
            fn checked(self, r: u8) -> Option<Self> { self.checked_shr(r as u32) }
        }
        )+
    }
}

impl_shr!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

pub trait Neg: Primary {
    fn wrapping(self) -> Self;
    fn saturating(self) -> Self;
    fn checked(self) -> Option<Self>;
}

macro_rules! impl_neg {
    ($($t:ty),+) => {
        $(
        impl Neg for $t {
            fn wrapping(self) -> Self { self.wrapping_neg() }
            fn saturating(self) -> Self { self.wrapping_neg() }
            fn checked(self) -> Option<Self> { self.checked_neg() }
        }
        )+
    }
}

impl_neg!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Neg for f32 {
    fn wrapping(self) -> Self { -self }

    fn saturating(self) -> Self { -self }

    fn checked(self) -> Option<Self> { Some(-self) }
}

impl Neg for f64 {
    fn wrapping(self) -> Self { -self }

    fn saturating(self) -> Self { -self }

    fn checked(self) -> Option<Self> { Some(-self) }
}

pub trait Inc: Primary {
    fn wrapping(self) -> Self;
    fn saturating(self) -> Self;
    fn checked(self) -> Option<Self>;
}

impl<T> Inc for T
    where
        T: Add,
{
    fn wrapping(self) -> Self { Add::wrapping(self, T::one()) }
    fn saturating(self) -> Self { Add::saturating(self, T::one()) }
    fn checked(self) -> Option<Self> { Add::checked(self, T::one()) }
}

pub trait Dec: Primary {
    fn wrapping(self) -> Self;
    fn saturating(self) -> Self;
    fn checked(self) -> Option<Self>;
}

impl<T> Dec for T
    where
        T: Sub,
{
    fn wrapping(self) -> Self { Sub::wrapping(self, T::one()) }
    fn saturating(self) -> Self { Sub::saturating(self, T::one()) }
    fn checked(self) -> Option<Self> { Sub::checked(self, T::one()) }
}

pub trait Convert<T>: Primary {
    fn convert(v: T) -> Self;
}

macro_rules! impl_convert {
    ($u:ty; $($t:ty),+) => {
        $(
        impl Convert<$t> for $u {
            fn convert(v: $t) -> Self {
                v as Self
            }
        }
        )+
    }
}

macro_rules! impl_convert_f {
    ($($u:ty),+) => {
        $(
        impl Convert<f32> for $u {
            fn convert(v: f32) -> Self {
                if v.is_nan() || v.is_infinite() {
                    <$u as Primary>::zero()
                } else {
                    v as Self
                }
            }
        }

        impl Convert<f64> for $u {
            fn convert(v: f64) -> Self {
                if v.is_nan() || v.is_infinite() {
                    <$u as Primary>::zero()
                } else {
                    v as Self
                }
            }
        }
        )+
    }
}

impl_convert_f!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64);
impl_convert!(u8; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(i8; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(u16; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(i16; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(u32; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(i32; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(u64; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(i64; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(u128; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(i128; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(usize; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(isize; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(f32; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_convert!(f64; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
