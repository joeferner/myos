#![no_std]

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
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
}

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            heap_start: 0,
            heap_end: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        self.heap_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}
