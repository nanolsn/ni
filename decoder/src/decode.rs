pub trait Decode<A>: Sized {
    type Err;

    fn decode<I>(bytes: &mut I, args: A) -> Result<Self, Self::Err>
        where
            I: Iterator<Item=u8>;
}

pub fn decode<T, I>(bytes: &mut I) -> Result<T, T::Err>
    where
        I: Iterator<Item=u8>,
        T: Decode<()>,
{ decode_with(bytes, ()) }

pub fn decode_with<T, I, N>(bytes: &mut I, args: N) -> Result<T, T::Err>
    where
        I: Iterator<Item=u8>,
        T: Decode<N>,
{ T::decode(bytes, args) }
