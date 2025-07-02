use core::alloc::Layout;

use crate::Allocator;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
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
    pub const unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

impl Allocator for BumpAllocator {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // TODO alignment and bounds check
        // TODO handle out of memory
        let alloc_start = self.next;
        self.next = alloc_start + layout.size();
        alloc_start as *mut u8
    }

    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
    }
}
