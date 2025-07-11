use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::AllocError;

use crate::Allocator;

struct Slab {
    next: Option<&'static mut Slab>,
}

type SlabSelectorFn = fn(&Layout) -> Option<usize>;
type SlabSizeFn = fn(usize) -> usize;

pub struct SlabAllocator<const SLAB_COUNT: usize, TFallback: Allocator> {
    slab_size_fn: SlabSizeFn,
    slab_selector_fn: SlabSelectorFn,
    slabs: [Option<&'static mut Slab>; SLAB_COUNT],
    fallback_allocator: TFallback,
}

impl<const SLAB_COUNT: usize, TFallback: Allocator> SlabAllocator<SLAB_COUNT, TFallback> {
    pub fn new(
        slab_size_fn: SlabSizeFn,
        slab_selector_fn: SlabSelectorFn,
        fallback_allocator: TFallback,
    ) -> Self {
        const EMPTY: Option<&'static mut Slab> = None;
        Self {
            slab_size_fn,
            slab_selector_fn,
            slabs: [EMPTY; SLAB_COUNT],
            fallback_allocator,
        }
    }

    /// Choose an slab for the given layout.
    ///
    /// Returns an index into the `slabs` array.
    fn list_index(&self, layout: &Layout) -> Option<usize> {
        (self.slab_selector_fn)(layout)
    }
}

impl<const SLAB_COUNT: usize, TFallback: Allocator> Allocator
    for SlabAllocator<SLAB_COUNT, TFallback>
{
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self.list_index(&layout) {
            Some(idx) => {
                match self.slabs[idx].take() {
                    Some(node) => {
                        self.slabs[idx] = node.next.take();
                        let ptr = node as *mut Slab as *mut u8;
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
                    }
                    None => {
                        // no block exists in list => allocate new block
                        let block_size = (self.slab_size_fn)(idx);
                        // only works if all block sizes are a power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        self.fallback_allocator.alloc(layout)
                    }
                }
            }
            None => self.fallback_allocator.alloc(layout),
        }
    }

    fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        match self.list_index(&layout) {
            Some(index) => {
                let new_node = Slab {
                    next: self.slabs[index].take(),
                };
                let new_node_ptr = ptr.as_ptr() as *mut Slab;
                unsafe {
                    new_node_ptr.write(new_node);
                    self.slabs[index] = Some(&mut *new_node_ptr);
                }
            }
            None => {
                self.fallback_allocator.dealloc(ptr, layout);
            }
        }
    }

    fn used(&self) -> usize {
        self.fallback_allocator.used()
    }

    fn free(&self) -> usize {
        self.fallback_allocator.free()
    }
}

#[cfg(test)]
mod tests {
    use core::alloc::Layout;

    use assert_hex::assert_eq_hex;

    use crate::{
        LinkedListAllocator,
        tests::{Memory, allocate},
    };

    use super::*;

    const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

    fn test_slab_size_fn(idx: usize) -> usize {
        BLOCK_SIZES[idx]
    }

    fn test_slab_selector_fn(layout: &Layout) -> Option<usize> {
        let required_block_size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
    }

    #[test]
    pub fn test_simple() {
        unsafe {
            const HEAP_SIZE: usize = 1000;
            let (heap_space_ptr, data_ptr) = Memory::<HEAP_SIZE>::new();

            let mut fallback_allocator = LinkedListAllocator::new();
            fallback_allocator.init(data_ptr, HEAP_SIZE);

            let mut allocator = SlabAllocator::<10, LinkedListAllocator>::new(
                test_slab_size_fn,
                test_slab_selector_fn,
                fallback_allocator,
            );

            let first_alloc = allocate(&mut allocator, Layout::new::<u32>()).unwrap();
            *first_alloc.as_mut_u32() = 0xdeadbeef;

            let second_alloc = allocate(&mut allocator, Layout::new::<u32>()).unwrap();
            *second_alloc.as_mut_u32() = 0xcafebabe;

            assert_eq_hex!(0xdeadbeef, *first_alloc.as_mut_u32());
            assert_eq_hex!(0xcafebabe, *second_alloc.as_mut_u32());
            let used = allocator.used();

            // verify that once memory is allocated to a slab it doesn't get released
            first_alloc.free(&mut allocator);
            assert_eq!(used, allocator.used());

            // verify that when allocating new data it uses the just freed slab part
            // also verify that the value is initialized to 0
            let third_alloc = allocate(&mut allocator, Layout::new::<u32>()).unwrap();
            assert_eq!(0, *third_alloc.as_mut_u32());
            *third_alloc.as_mut_u32() = 0xabadbabe;

            assert_eq_hex!(0xcafebabe, *second_alloc.as_mut_u32());
            assert_eq_hex!(0xabadbabe, *third_alloc.as_mut_u32());

            assert_eq!(used, allocator.used());

            second_alloc.free(&mut allocator);
            third_alloc.free(&mut allocator);

            assert_eq!(used, allocator.used());

            Memory::free(heap_space_ptr);
        }
    }
}
