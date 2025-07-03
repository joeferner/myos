use core::{alloc::Layout, usize};

use crate::Allocator;

pub struct BumpAllocator<const N: usize> {
    heap: [u8; N],
    next: usize,
}

impl<const N: usize> BumpAllocator<N> {
    pub const fn new() -> Self {
        BumpAllocator {
            heap: [0; N],
            next: 0,
        }
    }
}

impl<const N: usize> Allocator for BumpAllocator<N> {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // TODO alignment check
        if self.next.saturating_add(layout.size()) > self.heap.len() {
            return core::ptr::null_mut();
        }
        let heap = self.heap.as_ptr() as usize;
        let alloc_start = heap + self.next;
        self.next = self.next + layout.size();
        alloc_start as *mut u8
    }

    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {}
}
