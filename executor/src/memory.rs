use common::UWord;
use super::primary::Primary;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MemoryError {
    PageOverflow(&'static str),
    RageUnderflow(&'static str),
    SegmentationFault(UWord, UWord),
    WrongRange,
}

pub struct MemoryPage {
    page: Vec<u8>,
    limit: usize,
    name: &'static str,
}

impl MemoryPage {
    fn new(limit: usize, name: &'static str) -> Self {
        Self {
            page: Vec::new(),
            limit,
            name,
        }
    }

    pub fn expand(&mut self, size: UWord) -> Result<(), MemoryError> {
        let len = self.page.len().saturating_add(size as usize);

        if len > self.limit {
            Err(MemoryError::PageOverflow(self.name))
        } else {
            self.page.resize(len, 0);
            Ok(())
        }
    }

    pub fn narrow(&mut self, size: UWord) -> Result<(), MemoryError> {
        let size = size as usize;

        if self.page.len() < size {
            Err(MemoryError::PageOverflow(self.name))
        } else {
            let len = self.page.len() - size;
            self.page.truncate(len);
            Ok(())
        }
    }

    pub fn len(&self) -> UWord { self.page.len() as UWord }

    pub fn get(&self, ptr: UWord, size: UWord) -> Result<&[u8], MemoryError> {
        self.page.get(ptr as usize..ptr.wrapping_add(size) as usize)
            .ok_or(MemoryError::SegmentationFault(ptr, size))
    }

    pub fn get_mut(&mut self, ptr: UWord, size: UWord) -> Result<&mut [u8], MemoryError> {
        self.page.get_mut(ptr as usize..ptr.wrapping_add(size) as usize)
            .ok_or(MemoryError::SegmentationFault(ptr, size))
    }

    pub fn memmove(&mut self, dest: UWord, src: UWord, size: UWord) -> Result<(), MemoryError> {
        let src_end = src.wrapping_add(size);
        let dest_end = dest.wrapping_add(size);

        if src > src_end {
            Err(MemoryError::WrongRange)
        } else if src_end > self.len() {
            Err(MemoryError::SegmentationFault(src, size))
        } else if dest_end > self.len() {
            Err(MemoryError::SegmentationFault(dest, size))
        } else {
            self.page
                .as_mut_slice()
                .copy_within(src as usize..src_end as usize, dest as usize);

            Ok(())
        }
    }
}

impl std::fmt::Debug for MemoryPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        if self.page.is_empty() {
            return Ok(());
        }

        let mut counter = 0;
        let mut line = 0;

        f.write_char('\n')?;

        for &byte in self.page.iter() {
            if counter == 0 {
                write!(f, "{:02X?}:  ", line)?;
            }

            write!(f, "{:02X?} ", byte)?;
            counter += 1;
            line += 1;

            if counter > 8 {
                f.write_char('\n')?;
                counter = 0;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Memory {
    pub stack: MemoryPage,
    pub heap: MemoryPage,
}

impl Memory {
    pub const WORD_SIZE_BITS: UWord = std::mem::size_of::<UWord>() as UWord * 8;
    pub const HEAP_BASE: UWord = (1 as UWord) << (Self::WORD_SIZE_BITS / 2);

    pub fn from_limits(stack_limit: usize, heap_limit: usize) -> Self {
        if stack_limit >= Self::HEAP_BASE as usize {
            panic!("Stack limit must be less than heap base ({})", Self::HEAP_BASE)
        }

        Self {
            stack: MemoryPage::new(stack_limit, "stack"),
            heap: MemoryPage::new(heap_limit, "heap"),
        }
    }

    pub fn set<T>(&mut self, ptr: UWord, value: T) -> Result<(), MemoryError>
        where
            T: Primary,
    {
        use std::borrow::Borrow;

        let dest = self.slice_mut(ptr, T::SIZE as UWord)?;
        dest.copy_from_slice(value.to_bytes().borrow());
        Ok(())
    }

    pub fn get<T>(&self, ptr: UWord) -> Result<T, MemoryError>
        where
            T: Primary,
    {
        let src = self.slice(ptr, T::SIZE as UWord)?;
        Ok(T::from_slice(src))
    }

    pub fn update<T, F>(&mut self, ptr: UWord, f: F) -> Result<(), MemoryError>
        where
            T: Primary,
            F: FnOnce(T) -> T,
    { self.set(ptr, f(self.get(ptr)?)) }

    pub fn copy(&mut self, dest: UWord, src: UWord, size: UWord) -> Result<(), MemoryError> {
        let dest_on_stack = dest < Memory::HEAP_BASE;
        let src_on_stack = src < Memory::HEAP_BASE;

        // If dest and src are on the left or on the right side together then
        // they are in the same memory page.
        return if dest_on_stack == src_on_stack {
            // And then it allows to make a memmove.
            if dest_on_stack {
                self.stack.memmove(dest, src, size)
            } else {
                let dest = dest - Memory::HEAP_BASE;
                let src = src - Memory::HEAP_BASE;
                self.heap.memmove(dest, src, size)
            }
        } else {
            // Otherwise it requires to copy from one page to another.
            let (dest_slice, src_slice) = if dest_on_stack {
                let src = src - Memory::HEAP_BASE;
                (self.stack.get_mut(dest, size)?, self.heap.get(src, size)?)
            } else {
                let dest = dest - Memory::HEAP_BASE;
                (self.heap.get_mut(dest, size)?, self.stack.get(src, size)?)
            };

            dest_slice.copy_from_slice(src_slice);
            Ok(())
        };
    }

    pub fn set_zeros(&mut self, dest: UWord, size: UWord) -> Result<(), MemoryError> {
        let slice = self.slice_mut(dest, size)?;
        slice
            .iter_mut()
            .for_each(|b| *b = 0);

        Ok(())
    }

    pub fn compare(&self, a: UWord, b: UWord, size: UWord) -> Result<bool, MemoryError> {
        let a_slice = self.slice(a, size)?;
        let b_slice = self.slice(b, size)?;
        Ok(a_slice == b_slice)
    }

    fn slice(&self, ptr: UWord, size: UWord) -> Result<&[u8], MemoryError> {
        if ptr < Memory::HEAP_BASE {
            self.stack.get(ptr, size)
        } else {
            let ptr = ptr - Memory::HEAP_BASE;
            self.heap.get(ptr, size)
        }
    }

    fn slice_mut(&mut self, ptr: UWord, size: UWord) -> Result<&mut [u8], MemoryError> {
        if ptr < Memory::HEAP_BASE {
            self.stack.get_mut(ptr, size)
        } else {
            let ptr = ptr - Memory::HEAP_BASE;
            self.heap.get_mut(ptr, size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_append() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(4).unwrap();
        assert_eq!(mem.stack.len(), 4);
        assert_eq!(mem.stack.page.as_slice(), [0, 0, 0, 0]);

        let mut mem = Memory::from_limits(2048, 2048);
        assert_eq!(mem.stack.expand(UWord::MAX), Err(MemoryError::PageOverflow("stack")));
    }

    #[test]
    fn memory_set_get_stack() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(9).unwrap();
        mem.set(1, 0xFF000F0A_usize).unwrap();
        assert_eq!(mem.stack.page.as_slice(), [0, 10, 15, 0, 255, 0, 0, 0, 0]);

        let value: usize = mem.get(1).unwrap();
        assert_eq!(value, 0xFF000F0A_usize);
    }

    #[test]
    fn memory_set_get_heap() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.heap.expand(9).unwrap();
        mem.set(Memory::HEAP_BASE + 1, 0xFF000F0A_usize).unwrap();
        assert_eq!(mem.heap.page.as_slice(), [0, 10, 15, 0, 255, 0, 0, 0, 0]);

        let value: usize = mem.get(Memory::HEAP_BASE + 1).unwrap();
        assert_eq!(value, 0xFF000F0A_usize);
    }

    #[test]
    fn memory_update() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(8).unwrap();
        mem.set(0, 1).unwrap();

        mem.update(0, |v: u32| v + 1).unwrap();

        let value: usize = mem.get(0).unwrap();
        assert_eq!(value, 2);

        mem.update(0, |v: u32| v * v).unwrap();

        let value: usize = mem.get(0).unwrap();
        assert_eq!(value, 4);
    }

    #[test]
    fn memory_copy() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(8).unwrap();
        mem.set(0, 12).unwrap();

        mem.heap.expand(8).unwrap();
        mem.copy(Memory::HEAP_BASE, 0, 8).unwrap();

        assert_eq!(mem.stack.page.as_slice(), mem.heap.page.as_slice());

        let mut mem = Memory::from_limits(2048, 2048);
        mem.heap.expand(8).unwrap();
        mem.set(Memory::HEAP_BASE, 0xFF32).unwrap();

        mem.stack.expand(8).unwrap();
        mem.copy(0, Memory::HEAP_BASE, 8).unwrap();

        assert_eq!(mem.stack.page.as_slice(), mem.heap.page.as_slice());
    }

    #[test]
    fn memory_copy_move() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(16).unwrap();
        mem.set(0, 0xFF04).unwrap();

        mem.copy(8, 0, 8).unwrap();
        assert_eq!(mem.stack.page.as_slice(), [
            4, 255, 0, 0, 0, 0, 0, 0,
            4, 255, 0, 0, 0, 0, 0, 0,
        ]);

        let mut mem = Memory::from_limits(2048, 2048);
        mem.heap.expand(16).unwrap();
        mem.set(Memory::HEAP_BASE, 0xFF04).unwrap();

        mem.copy(Memory::HEAP_BASE + 8, Memory::HEAP_BASE, 8).unwrap();
        assert_eq!(mem.heap.page.as_slice(), [
            4, 255, 0, 0, 0, 0, 0, 0,
            4, 255, 0, 0, 0, 0, 0, 0,
        ]);
    }

    #[test]
    fn memory_copy_segmentation_fault() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(0).unwrap();
        mem.copy(0, 0, 0).unwrap();

        mem.stack.expand(1).unwrap();
        mem.copy(0, 0, 1).unwrap();
        mem.copy(1, 0, 0).unwrap();
        mem.copy(0, 1, 0).unwrap();
        mem.copy(1, 1, 0).unwrap();

        assert_eq!(mem.copy(1, 0, 1), Err(MemoryError::SegmentationFault(1, 1)));
        assert_eq!(mem.copy(0, 0, 2), Err(MemoryError::SegmentationFault(0, 2)));
    }

    #[test]
    fn memory_copy_wrong_range() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(2).unwrap();

        assert_eq!(mem.copy(0, 1, UWord::MAX), Err(MemoryError::WrongRange));
    }

    #[test]
    fn memory_set_zeros() {
        let mut mem = Memory::from_limits(2048, 2048);
        mem.stack.expand(16).unwrap();
        mem.set(0, 0xFFFF).unwrap();
        mem.set(8, 0xFFFF).unwrap();
        mem.set_zeros(0, 16).unwrap();

        assert!(mem.stack.page.iter().all(|b| *b == 0));

        let mut mem = Memory::from_limits(2048, 2048);
        mem.heap.expand(16).unwrap();
        mem.set(Memory::HEAP_BASE, 0xFFFF).unwrap();
        mem.set(Memory::HEAP_BASE + 8, 0xFFFF).unwrap();
        mem.set_zeros(Memory::HEAP_BASE, 16).unwrap();

        assert!(mem.heap.page.iter().all(|b| *b == 0));
    }
}
