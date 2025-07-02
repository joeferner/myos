use core::alloc::{GlobalAlloc, Layout};

use spin::Mutex;

use crate::{Allocator, BumpAllocator, LockedAllocator};

type AllocFn = fn(allocator: *const (), layout: Layout) -> *mut u8;
type DeallocFn = fn(allocator: *const (), ptr: *mut u8, layout: Layout);

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
        let z = allocator as *const T as usize;
        inner.allocator = z;
        let alloc = T::alloc as *const () as usize;
        inner.alloc = alloc;
        inner.dealloc = T::dealloc as *const () as usize;
    }
}

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let inner = self.inner.lock();
        if inner.allocator == 0 {
            return unsafe { FALLBACK_ALLOCATOR.alloc(layout) };
        }

        let allocator = inner.allocator as *const ();
        let alloc = inner.alloc as *const ();
        let alloc_fn: AllocFn = unsafe { core::mem::transmute(alloc) };
        alloc_fn(allocator, layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let inner = self.inner.lock();
        // TODO only call fallback dealloc if in fallback's memory range
        if inner.dealloc == 0 {
            return unsafe { FALLBACK_ALLOCATOR.dealloc(ptr, layout) };
        }

        let allocator = inner.allocator as *const ();
        let dealloc = inner.dealloc as *const ();
        let dealloc_fn: DeallocFn = unsafe { core::mem::transmute(dealloc) };
        dealloc_fn(allocator, ptr, layout)
    }
}
