use core::alloc::{GlobalAlloc, Layout};

use spin::Mutex;

use crate::{Allocator, BumpAllocator, LockedAllocator};

type AllocFn = fn(allocator: usize, layout: Layout) -> *mut u8;
type DeallocFn = fn(allocator: usize, ptr: *mut u8, layout: Layout);

const FALLBACK_TEST_MEMORY_SIZE: usize = 1 * 1024 * 1024;
static FALLBACK_ALLOCATOR: LockedAllocator<BumpAllocator<FALLBACK_TEST_MEMORY_SIZE>> =
    LockedAllocator::new(BumpAllocator::<FALLBACK_TEST_MEMORY_SIZE>::new());

pub struct TestAllocator {
    inner: Mutex<Inner>,
}

struct Inner {
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
            }),
        }
    }

    pub fn init<T: Allocator>(&self, allocator: &T) {
        let mut inner = self.inner.lock();
        inner.allocator = (allocator as *const T) as usize;
        inner.alloc = T::alloc as usize;
        inner.dealloc = T::dealloc as usize;
    }
}

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let inner = self.inner.lock();
        if inner.allocator == 0 {
            return unsafe { FALLBACK_ALLOCATOR.alloc(layout) };
        }

        let alloc_fn = inner.alloc as *const AllocFn;
        unsafe { (*alloc_fn)(inner.allocator, layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let inner = self.inner.lock();
        // TODO only call fallback dealloc if in fallback's memory range
        if inner.dealloc == 0 {
            return unsafe { FALLBACK_ALLOCATOR.dealloc(ptr, layout) };
        }

        let dealloc_fn = inner.dealloc as *const DeallocFn;
        unsafe { (*dealloc_fn)(inner.allocator, ptr, layout) }
    }
}
