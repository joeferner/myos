#![no_std]
#![feature(allocator_api)]

extern crate alloc;

mod linked_list_allocator;
mod locked_allocator;
mod slab_allocator;

use core::{alloc::Layout, ptr::NonNull};

pub use linked_list_allocator::LinkedListAllocator;
pub use locked_allocator::LockedAllocator;
pub use slab_allocator::SlabAllocator;

pub trait Allocator {
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError>;

    fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);

    fn used(&self) -> usize;
    fn free(&self) -> usize;
}

pub(crate) fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

#[cfg(test)]
mod tests {
    use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

    use alloc::{alloc::AllocError, boxed::Box};

    use crate::Allocator;

    #[repr(align(128))]
    pub struct Memory<const N: usize> {
        data: MaybeUninit<[u8; N]>,
    }

    impl<const N: usize> Memory<N> {
        /// Returns (almost certainly aliasing) pointers to the Memory
        /// as well as the data payload.
        ///
        /// MUST be freed with a matching call to `Memory::unleak`
        pub fn new() -> (*mut Memory<N>, *mut u8) {
            let heap_space_ptr: *mut Memory<N> = {
                let owned_box = Box::new(Self {
                    data: MaybeUninit::uninit(),
                });
                let mutref = Box::leak(owned_box);
                mutref
            };
            let data_ptr: *mut u8 =
                unsafe { core::ptr::addr_of_mut!((*heap_space_ptr).data).cast() };
            (heap_space_ptr, data_ptr)
        }

        pub unsafe fn free(putter: *mut Memory<N>) {
            drop(unsafe { Box::from_raw(putter) })
        }
    }

    pub struct Allocation(pub NonNull<[u8]>, pub Layout);

    impl Allocation {
        pub fn as_mut_u32(&self) -> *mut u32 {
            self.0.as_ptr() as *mut u32
        }

        pub fn free<T: Allocator>(self, allocator: &mut T) {
            let ptr = self.0.as_ptr() as *mut u8;
            let ptr = unsafe { NonNull::<u8>::new_unchecked(ptr) };
            allocator.dealloc(ptr, self.1);
        }
    }

    pub fn allocate<T: Allocator>(
        allocator: &mut T,
        layout: Layout,
    ) -> Result<Allocation, AllocError> {
        allocator.alloc(layout).map(|r| Allocation(r, layout))
    }
}
