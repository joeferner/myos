use core::alloc::{GlobalAlloc, Layout};

use spin::Mutex;

use crate::{Allocator, BumpAllocator};

type AllocFn = fn(allocator: usize, layout: Layout) -> *mut u8;
type DeallocFn = fn(allocator: usize, ptr: *mut u8, layout: Layout);

pub const FALLBACK_TEST_MEMORY_SIZE: usize = 1 * 1024 * 1024;
pub static FALLBACK_TEST_MEMORY: [u8; FALLBACK_TEST_MEMORY_SIZE] = [0; FALLBACK_TEST_MEMORY_SIZE];

pub struct TestAllocator {
    inner: Mutex<Inner>,
}

struct Inner {
    pub fallback_allocator: BumpAllocator,
    pub allocator: usize,
    pub alloc: usize,
    pub dealloc: usize,
}

impl TestAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                allocator: 0,
                alloc: 0,
                dealloc: 0,
                fallback_allocator: BumpAllocator::new(),
            }),
        }
    }

    pub fn init<T: Allocator>(&self, allocator: &T) {
        let mut inner = self.inner.lock();

        unsafe {
            inner.fallback_allocator.init(
                &FALLBACK_TEST_MEMORY as *const [u8; FALLBACK_TEST_MEMORY_SIZE] as usize,
                FALLBACK_TEST_MEMORY_SIZE,
            );
        }

        inner.allocator = (allocator as *const T) as usize;
        inner.alloc = T::alloc as usize;
        inner.dealloc = T::dealloc as usize;
    }
}

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = self.inner.lock();
        if inner.allocator == 0 {
            return inner.fallback_allocator.alloc(layout);
        }

        let alloc_fn = inner.alloc as *const AllocFn;
        unsafe { (*alloc_fn)(inner.allocator, layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let inner = self.inner.lock();
        let dealloc_fn = inner.dealloc as *const DeallocFn;
        unsafe { (*dealloc_fn)(inner.allocator, ptr, layout) }
    }
}
