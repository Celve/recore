use core::alloc::GlobalAlloc;

use spin::{Spin, SpinGuard};

use crate::buddy_allocator::BuddyAllocator;

pub struct LockedBuddyHeap {
    pub allocator: Spin<BuddyAllocator>,
}

impl LockedBuddyHeap {
    pub const fn empty(gran: usize) -> Self {
        Self {
            allocator: Spin::new(BuddyAllocator::empty(gran)),
        }
    }

    pub fn lock(&self) -> SpinGuard<BuddyAllocator> {
        self.allocator.lock()
    }

    pub unsafe fn add_segment(&self, start: usize, end: usize) {
        self.allocator.lock().add_segment(start, end);
    }
}

unsafe impl GlobalAlloc for LockedBuddyHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.allocator.lock().dealloc(ptr, layout);
    }
}
