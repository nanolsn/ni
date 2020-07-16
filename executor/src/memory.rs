use common::UWord;
use super::primary::Primary;

#[derive(Debug, Eq, PartialEq)]
pub enum MemoryError {
    PageOverflow(&'static str),
    RageUnderflow(&'static str),
    SegmentationFault(UWord, UWord),
    WrongRange,
    HeapAlreadyUsed,
}

const WORD_SIZE_BITS: usize = std::mem::size_of::<UWord>() * 8;

mod limits {
    use super::*;

    pub trait Limit {
        const LIMIT: usize;
        const NAME: &'static str;
    }

    #[derive(Debug)]
    pub struct Stack {}

    impl Limit for Stack {
        const LIMIT: usize = 1 << (WORD_SIZE_BITS / 3);
        const NAME: &'static str = "stack";
    }

    #[derive(Debug)]
    pub struct Heap {}

    impl Limit for Heap {
        const LIMIT: usize = 1 << (WORD_SIZE_BITS / 2);
        const NAME: &'static str = "heap";
    }
}

use limits::*;

pub struct MemoryPage<L> {
    page: Vec<u8>,
    limit: std::marker::PhantomData<L>,
}

impl<L> MemoryPage<L>
    where
        L: Limit,
{
    fn new() -> Self {
        Self {
            page: Vec::new(),
            limit: std::marker::PhantomData,
        }
    }

    pub fn expand(&mut self, size: UWord) -> Result<(), MemoryError> {
        let len = self.page.len().saturating_add(size as usize);

        if len > L::LIMIT {
            Err(MemoryError::PageOverflow(L::NAME))
        } else {
            self.page.resize(len, 0);
            Ok(())
        }
    }

    pub fn narrow(&mut self, size: UWord) -> Result<(), MemoryError> {
        let size = size as usize;

        if self.page.len() < size {
            Err(MemoryError::PageOverflow(L::NAME))
        } else {
            let len = self.page.len() - size;
            self.page.truncate(len);
            Ok(())
        }
    }

    pub fn len(&self) -> UWord { self.page.len() as UWord }

    pub fn as_slice(&self) -> &[u8] { self.page.as_slice() }

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

impl<L> std::fmt::Debug for MemoryPage<L>
    where
        L: Limit,
{
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
    pub stack: MemoryPage<Stack>,
    pub heap: MemoryPage<Heap>,
    pub global_base: UWord,
}

impl Memory {
    pub const HEAP_BASE: UWord = (1 as UWord) << (WORD_SIZE_BITS as UWord / 2);

    pub fn new() -> Self {
        Self {
            stack: MemoryPage::new(),
            heap: MemoryPage::new(),
            global_base: 0,
        }
    }

    pub fn reserve_global(&mut self, size: UWord) -> Result<(), MemoryError> {
        if self.heap.len() == 0 {
            self.heap.expand(size);
            self.global_base = size;
            Ok(())
        } else {
            Err(MemoryError::HeapAlreadyUsed)
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
        let mut mem = Memory::new();
        mem.stack.expand(4).unwrap();
        assert_eq!(mem.stack.len(), 4);
        assert_eq!(mem.stack.as_slice(), [0, 0, 0, 0]);

        let mut mem = Memory::new();
        assert_eq!(mem.stack.expand(UWord::MAX), Err(MemoryError::PageOverflow("stack")));
    }

    #[test]
    fn memory_set_get_stack() {
        let mut mem = Memory::new();
        mem.stack.expand(9).unwrap();
        mem.set(1, 0xFF000F0A_usize).unwrap();
        assert_eq!(mem.stack.as_slice(), [0, 10, 15, 0, 255, 0, 0, 0, 0]);

        let value: usize = mem.get(1).unwrap();
        assert_eq!(value, 0xFF000F0A_usize);
    }

    #[test]
    fn memory_set_get_heap() {
        let mut mem = Memory::new();
        mem.heap.expand(9).unwrap();
        mem.set(Memory::HEAP_BASE + 1, 0xFF000F0A_usize).unwrap();
        assert_eq!(mem.heap.as_slice(), [0, 10, 15, 0, 255, 0, 0, 0, 0]);

        let value: usize = mem.get(Memory::HEAP_BASE + 1).unwrap();
        assert_eq!(value, 0xFF000F0A_usize);
    }

    #[test]
    fn memory_update() {
        let mut mem = Memory::new();
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
        let mut mem = Memory::new();
        mem.stack.expand(8).unwrap();
        mem.set(0, 12).unwrap();

        mem.heap.expand(8).unwrap();
        mem.copy(Memory::HEAP_BASE, 0, 8).unwrap();

        assert_eq!(mem.stack.as_slice(), mem.heap.as_slice());

        let mut mem = Memory::new();
        mem.heap.expand(8).unwrap();
        mem.set(Memory::HEAP_BASE, 0xFF32).unwrap();

        mem.stack.expand(8).unwrap();
        mem.copy(0, Memory::HEAP_BASE, 8).unwrap();

        assert_eq!(mem.stack.as_slice(), mem.heap.as_slice());
    }

    #[test]
    fn memory_copy_move() {
        let mut mem = Memory::new();
        mem.stack.expand(16).unwrap();
        mem.set(0, 0xFF04).unwrap();

        mem.copy(8, 0, 8).unwrap();
        assert_eq!(mem.stack.as_slice(), [
            4, 255, 0, 0, 0, 0, 0, 0,
            4, 255, 0, 0, 0, 0, 0, 0,
        ]);

        let mut mem = Memory::new();
        mem.heap.expand(16).unwrap();
        mem.set(Memory::HEAP_BASE, 0xFF04).unwrap();

        mem.copy(Memory::HEAP_BASE + 8, Memory::HEAP_BASE, 8).unwrap();
        assert_eq!(mem.heap.as_slice(), [
            4, 255, 0, 0, 0, 0, 0, 0,
            4, 255, 0, 0, 0, 0, 0, 0,
        ]);
    }

    #[test]
    fn memory_copy_segmentation_fault() {
        let mut mem = Memory::new();
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
        let mut mem = Memory::new();
        mem.stack.expand(2).unwrap();

        assert_eq!(mem.copy(0, 1, UWord::MAX), Err(MemoryError::WrongRange));
    }

    #[test]
    fn memory_reserve_global() {
        let mut mem = Memory::new();
        mem.reserve_global(12).unwrap();
        mem.set::<u16>(Memory::HEAP_BASE, 0xFF32).unwrap();

        assert_eq!(mem.global_base, 12);
        let value = mem.get::<u16>(Memory::HEAP_BASE).unwrap();
        assert_eq!(value, 0xFF32);

        assert_eq!(mem.reserve_global(12), Err(MemoryError::HeapAlreadyUsed))
    }
}
