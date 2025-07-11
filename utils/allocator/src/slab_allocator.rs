use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::AllocError;

use crate::Allocator;

pub struct SlabAllocator<TFallback: Allocator> {
    fallback_allocator: TFallback,
}

impl<TFallback: Allocator> SlabAllocator<TFallback> {
    pub fn new(fallback_allocator: TFallback) -> Self {
        Self { fallback_allocator }
    }
}

impl<TFallback: Allocator> Allocator for SlabAllocator<TFallback> {
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.fallback_allocator.alloc(layout)
    }

    fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.fallback_allocator.dealloc(ptr, layout)
    }
}

#[cfg(test)]
mod tests {
    use core::alloc::Layout;

    use assert_hex::assert_eq_hex;

    use crate::{LinkedListAllocator, tests::Memory};

    use super::*;

    #[test]
    pub fn test_simple() {
        unsafe {
            const HEAP_SIZE: usize = 1000;
            let (heap_space_ptr, data_ptr) = Memory::<HEAP_SIZE>::new();

            let mut fallback_allocator = LinkedListAllocator::new();
            fallback_allocator.init(data_ptr, HEAP_SIZE);

            let mut alloc = SlabAllocator::new(fallback_allocator);

            let first_alloc = {
                let m = alloc.alloc(Layout::new::<u32>()).unwrap().as_ptr() as *mut u32;
                *m = 0xdeadbeef;
                m
            };

            let second_alloc = {
                let m = alloc.alloc(Layout::new::<u32>()).unwrap().as_ptr() as *mut u32;
                *m = 0xcafebabe;
                m
            };

            assert_eq_hex!(0xdeadbeef, *first_alloc);
            assert_eq_hex!(0xcafebabe, *second_alloc);

            Memory::free(heap_space_ptr);
        }
    }
}
