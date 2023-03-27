use core::alloc::GlobalAlloc;

use spin::mutex::Mutex;

use crate::buddy_allocator::BuddyAllocator;

pub struct LockedHeap {
    pub allocator: Mutex<BuddyAllocator>,
}

impl LockedHeap {
    pub const fn empty(gran: usize) -> Self {
        Self {
            allocator: Mutex::new(BuddyAllocator::empty(gran)),
        }
    }

    pub unsafe fn add_segment(&self, start: usize, end: usize) {
        self.allocator.lock().add_segment(start, end);
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.allocator.lock().dealloc(ptr, layout);
    }
}
