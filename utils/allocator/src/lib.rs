#![no_std]

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
// TODO use core::ptr::null_mut;
use spin::Mutex;

pub struct LockedAllocator {
    inner: Mutex<Allocator>,
}

impl LockedAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(Allocator::new()),
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        unsafe { self.inner.lock().init(heap_start, heap_size) }
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.inner.lock().alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.inner.lock().dealloc(ptr, layout) }
    }
}

pub struct Allocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // TODO alignment and bounds check
        let alloc_start = self.next;
        self.next = alloc_start + layout.size();
        alloc_start as *mut u8
    }

    pub unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
    }
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;

    use super::*;

    #[test]
    pub fn test_simple() {
        unsafe {
            const HEAP_SIZE: usize = 100;
            let heap: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
            let mut alloc = Allocator::new();
            alloc.init(&heap as *const [u8; HEAP_SIZE] as usize, HEAP_SIZE);

            let first_alloc = {
                let m = alloc.alloc(Layout::new::<u32>()) as *mut u32;
                *m = 0xdeadbeef;
                m
            };

            let second_alloc = {
                let m = alloc.alloc(Layout::new::<u32>()) as *mut u32;
                *m = 0xcafebabe;
                m
            };

            assert_eq_hex!(0xdeadbeef, *first_alloc);
            assert_eq_hex!(0xcafebabe, *second_alloc);
        }
    }
}
