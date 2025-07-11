use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::AllocError;

use crate::{Allocator, is_power_of_two};

struct BlockNode {
    next: Option<&'static mut BlockNode>,
}

type SlabSelectorFn = fn(&Layout) -> Option<usize>;
type BlockSizeFn = fn(usize) -> usize;

pub struct SlabAllocator<const SLAB_COUNT: usize, TFallback: Allocator> {
    block_size_fn: BlockSizeFn,
    slab_selector_fn: SlabSelectorFn,
    slabs: [Option<&'static mut BlockNode>; SLAB_COUNT],
    fallback_allocator: TFallback,
    page_size: usize,
}

/// The block sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
impl<const SLAB_COUNT: usize, TFallback: Allocator> SlabAllocator<SLAB_COUNT, TFallback> {
    pub fn new(
        block_size_fn: BlockSizeFn,
        slab_selector_fn: SlabSelectorFn,
        fallback_allocator: TFallback,
        page_size: usize,
    ) -> Self {
        for i in 0..SLAB_COUNT {
            let black_size = block_size_fn(i);
            if !is_power_of_two(black_size) {
                assert!(false);
            }
            assert!(core::mem::size_of::<BlockNode>() <= black_size);
            assert!(core::mem::align_of::<BlockNode>() <= black_size);
        }
        const EMPTY: Option<&'static mut BlockNode> = None;
        Self {
            block_size_fn,
            slab_selector_fn,
            slabs: [EMPTY; SLAB_COUNT],
            fallback_allocator,
            page_size,
        }
    }

    /// Choose an slab for the given layout.
    ///
    /// Returns an index into the `slabs` array.
    fn slab_index(&self, layout: &Layout) -> Option<usize> {
        (self.slab_selector_fn)(layout)
    }

    fn allocate_new_blocks_in_slab(&mut self, slab_idx: usize) -> Result<(), AllocError> {
        let block_size = (self.block_size_fn)(slab_idx);
        let layout = Layout::from_size_align(block_size, block_size).unwrap();
        let block_count = self.page_size / block_size;
        for _ in 0..block_count {
            let ptr = self.fallback_allocator.alloc(layout)?;
            let new_node_ptr = ptr.as_ptr() as *mut BlockNode;
            let new_node = BlockNode {
                next: self.slabs[slab_idx].take(),
            };
            unsafe {
                new_node_ptr.write(new_node);
                self.slabs[slab_idx] = Some(&mut *new_node_ptr);
            }
        }
        Ok(())
    }
}

impl<const SLAB_COUNT: usize, TFallback: Allocator> Allocator
    for SlabAllocator<SLAB_COUNT, TFallback>
{
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self.slab_index(&layout) {
            Some(slab_idx) => {
                // no block exists in list => allocate new block
                if self.slabs[slab_idx].is_none() {
                    self.allocate_new_blocks_in_slab(slab_idx)?;
                }

                match self.slabs[slab_idx].take() {
                    Some(node) => {
                        self.slabs[slab_idx] = node.next.take();
                        let ptr = node as *mut BlockNode as *mut u8;
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
                    }
                    None => Err(AllocError),
                }
            }
            None => self.fallback_allocator.alloc(layout),
        }
    }

    fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        match self.slab_index(&layout) {
            Some(index) => {
                let new_node = BlockNode {
                    next: self.slabs[index].take(),
                };
                let new_node_ptr = ptr.as_ptr() as *mut BlockNode;
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

    const BLOCK_SIZES: &[usize] = &[16, 32, 64, 128, 256, 512, 1024, 2048];
    const SLAB_COUNT: usize = BLOCK_SIZES.len();

    fn test_block_size_fn(idx: usize) -> usize {
        BLOCK_SIZES[idx]
    }

    fn test_slab_selector_fn(layout: &Layout) -> Option<usize> {
        let required_block_size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
    }

    #[test]
    pub fn test_simple() {
        unsafe {
            const PAGE_SIZE: usize = 2048;
            const HEAP_SIZE: usize = 2048;
            let (heap_space_ptr, data_ptr) = Memory::<HEAP_SIZE>::new();

            let mut fallback_allocator = LinkedListAllocator::new();
            fallback_allocator.init(data_ptr, HEAP_SIZE);

            let mut allocator = SlabAllocator::<SLAB_COUNT, LinkedListAllocator>::new(
                test_block_size_fn,
                test_slab_selector_fn,
                fallback_allocator,
                PAGE_SIZE,
            );

            let first_alloc = allocate(&mut allocator, Layout::new::<u32>()).unwrap();
            *first_alloc.as_mut_u32() = 0xdeadbeef;

            let second_alloc = allocate(&mut allocator, Layout::new::<u32>()).unwrap();
            *second_alloc.as_mut_u32() = 0xcafebabe;

            assert_eq_hex!(0xdeadbeef, *first_alloc.as_mut_u32());
            assert_eq_hex!(0xcafebabe, *second_alloc.as_mut_u32());
            assert_eq!(PAGE_SIZE, allocator.used());

            // verify that once memory is allocated to a slab it doesn't get released
            first_alloc.free(&mut allocator);
            assert_eq!(PAGE_SIZE, allocator.used());

            // verify that when allocating new data it uses the just freed slab part
            // also verify that the value is initialized to 0
            let third_alloc = allocate(&mut allocator, Layout::new::<u32>()).unwrap();
            assert_eq!(0, *third_alloc.as_mut_u32());
            *third_alloc.as_mut_u32() = 0xabadbabe;

            assert_eq_hex!(0xcafebabe, *second_alloc.as_mut_u32());
            assert_eq_hex!(0xabadbabe, *third_alloc.as_mut_u32());

            assert_eq!(PAGE_SIZE, allocator.used());

            second_alloc.free(&mut allocator);
            third_alloc.free(&mut allocator);

            assert_eq!(PAGE_SIZE, allocator.used());

            Memory::free(heap_space_ptr);
        }
    }
}
