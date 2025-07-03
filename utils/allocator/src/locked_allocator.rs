extern crate alloc;

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

use crate::Allocator;
use alloc::alloc::AllocError;
use core::ptr::null_mut;
use spin::Mutex;

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
        match self.inner.lock().alloc(layout) {
            Ok(p) => p.as_ptr() as *mut u8,
            Err(_) => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        self.inner.lock().dealloc(ptr, layout)
    }
}

unsafe impl<T: Allocator> core::alloc::Allocator for LockedAllocator<T> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.inner.lock().alloc(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }
}
