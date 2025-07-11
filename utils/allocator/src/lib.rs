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
}

#[cfg(test)]
mod tests {
    use core::mem::MaybeUninit;

    use alloc::boxed::Box;

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
}
