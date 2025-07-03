use alloc::alloc::Layout;

use crate::Allocator;
// TODO use core::ptr::null_mut;

pub struct LinkedListAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// # Safety
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

impl Allocator for LinkedListAllocator {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // TODO alignment and bounds check
        let alloc_start = self.next;
        self.next = alloc_start + layout.size();
        alloc_start as *mut u8
    }

    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::tests::{TEST_ALLOCATOR, TEST_MEMORY, TEST_MEMORY_SIZE};

    use super::*;

    #[test]
    #[allow(static_mut_refs)]
    pub fn test_simple() {
        let mut allocator = LinkedListAllocator::new();
        unsafe {
            TEST_MEMORY[0] = 0xde;
            TEST_MEMORY[1] = 0xad;
            allocator.init(TEST_MEMORY.as_ptr() as usize, TEST_MEMORY_SIZE);
        }
        TEST_ALLOCATOR.init(&allocator);
        {
            let b = Box::new(42);
            assert_eq!(42, *b);
        }
        TEST_ALLOCATOR.uninit();
    }
}
