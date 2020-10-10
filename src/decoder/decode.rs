pub trait Decode<A>: Sized {
    type Err;

    fn decode<R>(bytes: &mut R, args: A) -> Result<Self, Self::Err>
    where
        R: std::io::Read;
}

pub fn decode<T, R>(bytes: &mut R) -> Result<T, T::Err>
where
    R: std::io::Read,
    T: Decode<()>,
{
    decode_with(bytes, ())
}

pub fn decode_with<T, R, N>(bytes: &mut R, args: N) -> Result<T, T::Err>
where
    R: std::io::Read,
    T: Decode<N>,
{
    T::decode(bytes, args)
}
