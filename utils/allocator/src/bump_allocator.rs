use core::{alloc::Layout, ptr::NonNull, usize};

use alloc::alloc::AllocError;

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
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // TODO alignment check
        if self.next.saturating_add(layout.size()) > self.heap.len() {
            return Err(AllocError);
        }
        let heap = self.heap.as_ptr() as usize;
        let alloc_start = heap + self.next;
        self.next = self.next + layout.size();
        let slice: *mut [u8] =
            unsafe { core::slice::from_raw_parts_mut(alloc_start as *mut u8, layout.size()) };
        Ok(NonNull::new(slice).unwrap())
    }

    fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
}
