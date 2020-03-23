#[derive(Debug)]
pub struct ByteIterator<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ByteIterator<'a> {
    pub fn new(bytes: &'a [u8]) -> Self { ByteIterator { bytes, pos: 0 } }

    pub fn get(&self) -> Option<u8> { self.bytes.get(self.pos).map(|&p| p) }

    pub fn pos(&self) -> usize { self.pos }

    pub fn end(&self) -> bool { self.bytes.len() == self.pos }
}

impl<'a> Iterator for ByteIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.get();

        if res.is_some() {
            self.pos += 1;
        }

        res
    }
}
