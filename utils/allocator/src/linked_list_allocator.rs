use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::AllocError;
use linked_list_allocator::Heap;

use crate::Allocator;

pub struct LinkedListAllocator {
    heap: Heap,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            heap: Heap::empty(),
        }
    }
}

impl Allocator for LinkedListAllocator {
    unsafe fn init(&mut self, data_ptr: *mut u8, heap_size: usize) {
        unsafe { self.heap.init(data_ptr, heap_size) }
    }

    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.heap
            .allocate_first_fit(layout)
            .map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size()))
            .map_err(|_| AllocError)
    }

    fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.heap.deallocate(ptr, layout);
        }
    }

    fn used(&self) -> usize {
        self.heap.used()
    }

    fn free(&self) -> usize {
        self.heap.free()
    }
}
