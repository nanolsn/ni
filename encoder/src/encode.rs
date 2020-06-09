pub trait Encode {
    type Err;

    fn encode<I>(&self) -> Result<I, Self::Err>
        where
            I: Iterator<Item=u8>;
}
