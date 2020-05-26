pub trait Decode: Sized {
    type Err;

    fn decode<I>(bytes: &mut I) -> Option<Result<Self, Self::Err>>
        where
            I: Iterator<Item=u8>;
}
