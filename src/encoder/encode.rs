pub trait Encode {
    type Err;

    fn encode<W>(&self, buf: &mut W) -> Result<(), Self::Err>
        where
            W: std::io::Write;
}
