#![no_std]

extern crate alloc;

mod linked_list_allocator;
mod locked_allocator;
mod bump_allocator;
#[cfg(test)]
mod test_allocator;

use core::alloc::Layout;

pub use linked_list_allocator::LinkedListAllocator;
pub use locked_allocator::LockedAllocator;
pub use bump_allocator::BumpAllocator;

pub trait Allocator {
    fn alloc(&mut self, layout: Layout) -> *mut u8;
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
}

#[cfg(test)]
mod tests {
    use crate::test_allocator::TestAllocator;

    #[global_allocator]
    pub static TEST_ALLOCATOR: TestAllocator = TestAllocator::new();

    pub const TEST_MEMORY_SIZE: usize = 100000;
    pub static mut TEST_MEMORY: [u8; TEST_MEMORY_SIZE] = [0; TEST_MEMORY_SIZE];

    // #[test]
    // pub fn test_simple() {
    //     unsafe {
    //         const HEAP_SIZE: usize = 100;
    //         let heap: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    //         let mut alloc = LinkedListAllocator::new();
    //         alloc.init(&heap as *const [u8; HEAP_SIZE] as usize, HEAP_SIZE);

    //         let first_alloc = {
    //             let m = alloc.alloc(Layout::new::<u32>()) as *mut u32;
    //             *m = 0xdeadbeef;
    //             m
    //         };

    //         let second_alloc = {
    //             let m = alloc.alloc(Layout::new::<u32>()) as *mut u32;
    //             *m = 0xcafebabe;
    //             m
    //         };

    //         assert_eq_hex!(0xdeadbeef, *first_alloc);
    //         assert_eq_hex!(0xcafebabe, *second_alloc);
    //     }
    // }
}
