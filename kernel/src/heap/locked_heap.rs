use core::alloc::GlobalAlloc;

use allocator::buddy_allocator::BuddyAllocator;
use spin::Spin;

use crate::config::KERNEL_HEAP_GRANULARITY;

use super::slab_allocator::SlabAllocator;

pub struct LockedHeap {
    pub slab_allocator: Spin<SlabAllocator>,
    pub buddy_allocator: Spin<BuddyAllocator>,
}

impl LockedHeap {
    pub const fn empty() -> Self {
        Self {
            slab_allocator: Spin::new(SlabAllocator::empty()),
            buddy_allocator: Spin::new(BuddyAllocator::empty(KERNEL_HEAP_GRANULARITY)),
        }
    }

    pub unsafe fn init(&self, start: usize, end: usize) {
        self.slab_allocator.lock().init();
        self.buddy_allocator.lock().add_segment(start, end);
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.size() < 4096 {
            self.slab_allocator.lock().alloc(layout)
        } else {
            self.buddy_allocator.lock().alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if layout.size() < 4096 {
            self.slab_allocator.lock().dealloc(ptr, layout);
        } else {
            self.buddy_allocator.lock().dealloc(ptr, layout);
        }
    }
}
