pub trait ExpectedError {
    const ERROR: Self;
}

pub trait Expected {
    fn expected<E>(self, bytes: usize) -> Result<(), E>
        where
            E: ExpectedError + From<std::io::Error>;
}

impl Expected for Result<usize, std::io::Error> {
    fn expected<E>(self, bytes: usize) -> Result<(), E>
        where
            E: ExpectedError + From<std::io::Error>,
    {
        match self {
            Ok(r) if r == bytes => Ok(()),
            Ok(_) => Err(E::ERROR),
            Err(e) => Err(e.into()),
        }
    }
}
