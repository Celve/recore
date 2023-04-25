use core::alloc::GlobalAlloc;

use allocator::buddy_allocator::BuddyAllocator;
use spin::Spin;

use crate::config::KERNEL_HEAP_GRANULARITY;

use super::slab_allocator::SlabAllocator;

pub struct Heap {
    pub slab_allocator: SlabAllocator,
    pub buddy_allocator: Spin<BuddyAllocator>,
}

impl Heap {
    pub const fn default() -> Self {
        Self {
            slab_allocator: SlabAllocator::empty(),
            buddy_allocator: Spin::new(BuddyAllocator::empty(KERNEL_HEAP_GRANULARITY)),
        }
    }
}

impl Heap {
    pub unsafe fn init(&self, start: usize, end: usize) {
        self.slab_allocator.init();
        self.buddy_allocator.lock().add_segment(start, end);
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.size() < 4096 {
            self.slab_allocator.alloc(layout)
        } else {
            self.buddy_allocator.lock().alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if layout.size() < 4096 {
            self.slab_allocator.dealloc(ptr, layout);
        } else {
            self.buddy_allocator.lock().dealloc(ptr, layout);
        }
    }
}
