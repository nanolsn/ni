pub trait Primary: Sized {
    const SIZE: usize = std::mem::size_of::<Self>();

    type Bytes: std::borrow::Borrow<[u8]>;

    fn to_bytes(&self) -> Self::Bytes;

    fn from_bytes(bytes: Self::Bytes) -> Self;
}

macro_rules! impl_primary {
    ($($t:ty),+) => {
        $(
        impl Primary for $t {
            type Bytes = [u8; std::mem::size_of::<Self>()];

            fn to_bytes(&self) -> Self::Bytes { self.to_ne_bytes() }

            fn from_bytes(bytes: Self::Bytes) -> Self { Self::from_ne_bytes(bytes) }
        }
        )+
    }
}

impl_primary!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, f32, f64);
