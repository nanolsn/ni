#[derive(Debug, Eq, PartialEq)]
pub enum MemoryError {
    StackOverflow,
    HeapOverflow,
    SegmentationFault,
}

#[derive(Debug)]
pub struct Memory {
    stack: Vec<u8>,
    heap: Vec<u8>,
}

impl Memory {
    pub const STACK_LIMIT: usize = 1_usize << 23;
    pub const HEAP_LIMIT: usize = 1_usize << 31;
    pub const HEAP_BASE: usize = 1_usize << 32;

    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            heap: Vec::new(),
        }
    }

    pub fn mem(&self, ptr: usize, size: usize) -> Option<&[u8]> {
        if ptr <= Memory::HEAP_BASE {
            self.stack.get(ptr..ptr.wrapping_add(size))
        } else {
            let ptr = ptr - Memory::HEAP_BASE;
            self.heap.get(ptr..ptr.wrapping_add(size))
        }
    }

    pub fn mut_mem(&mut self, ptr: usize, size: usize) -> Option<&mut [u8]> {
        if ptr <= Memory::HEAP_BASE {
            self.stack.get_mut(ptr..ptr.wrapping_add(size))
        } else {
            let ptr = ptr - Memory::HEAP_BASE;
            self.heap.get_mut(ptr..ptr.wrapping_add(size))
        }
    }

    pub fn append_stack(&mut self, size: usize) -> Result<(), MemoryError> {
        let len = self.stack.len().wrapping_add(size);

        if len > Memory::STACK_LIMIT {
            Err(MemoryError::StackOverflow)
        } else {
            self.stack.resize(len, 0);
            Ok(())
        }
    }

    pub fn append_heap(&mut self, size: usize) -> Result<(), MemoryError> {
        let len = self.stack.len().wrapping_add(size);

        if len > Memory::HEAP_LIMIT {
            Err(MemoryError::HeapOverflow)
        } else {
            self.heap.resize(len, 0);
            Ok(())
        }
    }

    pub fn set(&mut self, ptr: usize, value: usize) -> Result<(), MemoryError> {
        let bytes: [u8; std::mem::size_of::<usize>()] = value.to_ne_bytes();

        self.mut_mem(ptr, bytes.len())
            .ok_or(MemoryError::SegmentationFault)
            .map(|s| {
                s.copy_from_slice(&bytes);
                ()
            })
    }

    pub fn get(&self, ptr: usize) -> Result<usize, MemoryError> {
        use std::convert::TryInto;

        const SIZE: usize = std::mem::size_of::<usize>();

        self.mem(ptr, SIZE)
            .ok_or(MemoryError::SegmentationFault)
            .map(|s| usize::from_ne_bytes(s.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_append() {
        let mut mem = Memory::new();
        assert!(mem.append_stack(4).is_ok());
        assert_eq!(mem.stack.len(), 4);
        assert_eq!(mem.stack.as_slice(), [0, 0, 0, 0]);

        let mut mem = Memory::new();
        assert_eq!(mem.append_stack(usize::MAX), Err(MemoryError::StackOverflow));
    }

    #[test]
    fn memory_set() {
        let mut mem = Memory::new();
        assert!(mem.append_stack(9).is_ok());
        assert!(mem.set(1, 0xFF000F0A).is_ok());
        assert_eq!(mem.stack, [0, 10, 15, 0, 255, 0, 0, 0, 0]);

        let value = mem.get(1).unwrap();
        assert_eq!(value, 0xFF000F0A);
    }

    #[test]
    fn memory_set_heap() {
        let mut mem = Memory::new();
        assert!(mem.append_heap(9).is_ok());
        assert!(mem.set(Memory::HEAP_BASE + 1, 0xFF000F0A).is_ok());
        assert_eq!(mem.heap, [0, 10, 15, 0, 255, 0, 0, 0, 0]);

        let value = mem.get(Memory::HEAP_BASE + 1).unwrap();
        assert_eq!(value, 0xFF000F0A);
    }
}
