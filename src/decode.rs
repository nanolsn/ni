pub trait Decode: Sized {
    type Err;

    fn decode<I>(bytes: &mut I) -> Option<Result<Self, Self::Err>>
        where
            I: Iterator<Item=u8>;
}

pub trait FromByte: Sized {
    type Err;

    fn from_byte(byte: u8) -> Result<Self, Self::Err>;
}

impl<T> Decode for T
    where
        T: FromByte,
{
    type Err = T::Err;

    fn decode<I>(bytes: &mut I) -> Option<Result<Self, Self::Err>>
        where
            I: Iterator<Item=u8>,
    { Some(T::from_byte(bytes.next()?)) }
}
