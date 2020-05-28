use crate::Primary;

#[derive(Debug, Eq, PartialEq)]
pub enum MemoryError {
    PageOverflow(&'static str),
    RageUnderflow(&'static str),
    SegmentationFault,
    WrongRange,
}

mod limits {
    pub trait Limit {
        const LIMIT: usize;
        const NAME: &'static str;
    }

    #[derive(Debug)]
    pub struct Stack {}

    impl Limit for Stack {
        const LIMIT: usize = 1_usize << 23;
        const NAME: &'static str = "stack";
    }

    #[derive(Debug)]
    pub struct Heap {}

    impl Limit for Heap {
        const LIMIT: usize = 1_usize << 31;
        const NAME: &'static str = "heap";
    }
}

use limits::*;

#[derive(Debug)]
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

    pub fn expand(&mut self, size: usize) -> Result<(), MemoryError> {
        let len = self.page.len().saturating_add(size);

        if len > L::LIMIT {
            Err(MemoryError::PageOverflow(L::NAME))
        } else {
            self.page.resize(len, 0);
            Ok(())
        }
    }

    pub fn narrow(&mut self, size: usize) -> Result<(), MemoryError> {
        if self.page.len() < size {
            Err(MemoryError::PageOverflow(L::NAME))
        } else {
            let len = self.page.len() - size;
            self.page.truncate(len);
            Ok(())
        }
    }

    pub fn len(&self) -> usize { self.page.len() }

    pub fn as_slice(&self) -> &[u8] { self.page.as_slice() }

    pub fn get(&self, ptr: usize, size: usize) -> Option<&[u8]> {
        self.page.get(ptr..ptr.wrapping_add(size))
    }

    pub fn get_mut(&mut self, ptr: usize, size: usize) -> Option<&mut [u8]> {
        self.page.get_mut(ptr..ptr.wrapping_add(size))
    }

    pub fn memmove(&mut self, dest: usize, src: usize, size: usize) -> Result<(), MemoryError> {
        let src_end = src.wrapping_add(size);
        let dest_end = dest.wrapping_add(size);

        if src > src_end {
            Err(MemoryError::WrongRange)
        } else if src_end.max(dest_end) > self.len() {
            Err(MemoryError::SegmentationFault)
        } else {
            self.page
                .as_mut_slice()
                .copy_within(src..src_end, dest);

            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    pub stack: MemoryPage<Stack>,
    pub heap: MemoryPage<Heap>,
}

impl Memory {
    pub const HEAP_BASE: usize = 1_usize << 32;

    pub fn new() -> Self {
        Self {
            stack: MemoryPage::new(),
            heap: MemoryPage::new(),
        }
    }

    pub fn set<T>(&mut self, ptr: usize, value: T) -> Result<(), MemoryError>
        where
            T: Primary,
    {
        use std::borrow::Borrow;

        self.slice_mut(ptr, T::SIZE)
            .ok_or(MemoryError::SegmentationFault)
            .map(|s| {
                s.copy_from_slice(value.to_bytes().borrow());
                ()
            })
    }

    pub fn get<T>(&self, ptr: usize) -> Result<T, MemoryError>
        where
            T: Primary,
    {
        self.slice(ptr, T::SIZE)
            .ok_or(MemoryError::SegmentationFault)
            .map(|sl| T::from_slice(sl))
    }

    pub fn update<T, F>(&mut self, ptr: usize, f: F) -> Result<(), MemoryError>
        where
            T: Primary,
            F: FnOnce(T) -> T,
    { self.set(ptr, f(self.get(ptr)?)) }

    pub fn copy(&mut self, dest: usize, src: usize, size: usize) -> Result<(), MemoryError> {
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
                (self.stack.get_mut(dest, size), self.heap.get(src, size))
            } else {
                let dest = dest - Memory::HEAP_BASE;
                (self.heap.get_mut(dest, size), self.stack.get(src, size))
            };

            dest_slice
                .and_then(|d| src_slice.map(|s| d.copy_from_slice(s)))
                .ok_or(MemoryError::SegmentationFault)
        };
    }

    fn slice(&self, ptr: usize, size: usize) -> Option<&[u8]> {
        if ptr < Memory::HEAP_BASE {
            self.stack.get(ptr, size)
        } else {
            let ptr = ptr - Memory::HEAP_BASE;
            self.heap.get(ptr, size)
        }
    }

    fn slice_mut(&mut self, ptr: usize, size: usize) -> Option<&mut [u8]> {
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
        assert_eq!(mem.stack.expand(usize::MAX), Err(MemoryError::PageOverflow("stack")));
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

        assert_eq!(mem.copy(1, 0, 1), Err(MemoryError::SegmentationFault));
        assert_eq!(mem.copy(0, 0, 2), Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn memory_copy_wrong_range() {
        let mut mem = Memory::new();
        mem.stack.expand(2).unwrap();

        assert_eq!(mem.copy(0, 1, usize::MAX), Err(MemoryError::WrongRange));
    }
}
