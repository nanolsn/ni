pub trait Primary: Sized {
    const SIZE: usize = std::mem::size_of::<Self>();

    type Bytes: std::borrow::Borrow<[u8]>;

    fn to_bytes(&self) -> Self::Bytes;

    fn from_bytes(bytes: Self::Bytes) -> Self;

    fn from_slice(slice: &[u8]) -> Self;

    fn zero() -> Self;

    fn from_usize(val: usize) -> Self;
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

            fn from_usize(val: usize) -> Self {
                let mut buf = [0; Self::SIZE];
                let ubytes = usize::to_le_bytes(val);

                for i in 0..Self::SIZE.min(std::mem::size_of::<usize>()) {
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
