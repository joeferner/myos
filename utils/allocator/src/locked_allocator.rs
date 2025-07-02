extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

use crate::Allocator;

pub struct LockedAllocator<T: Allocator> {
    inner: Mutex<T>,
}

impl<T: Allocator> LockedAllocator<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new(allocator: T) -> Self {
        Self {
            inner: Mutex::new(allocator),
        }
    }
}

unsafe impl<T: Allocator> GlobalAlloc for LockedAllocator<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }
}
