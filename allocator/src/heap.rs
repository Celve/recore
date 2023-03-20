use core::{alloc::GlobalAlloc, cell::RefCell};

use crate::buddy_allocator::BuddyAllocator;

pub struct FakeMutex<T> {
    pub inner: RefCell<T>,
}

pub struct LockedHeap {
    pub allocator: FakeMutex<BuddyAllocator>,
}

impl<T> FakeMutex<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }
}

impl LockedHeap {
    pub const fn new(gran: usize) -> Self {
        Self {
            allocator: FakeMutex::new(BuddyAllocator::empty(gran)),
        }
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator.inner.borrow_mut().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.allocator.inner.borrow_mut().dealloc(ptr, layout);
    }
}

/// It's unsafe to do such a thing here. Because it's not thread-safe without a mutex.
/// I will fix it up when I implement the mutex.
unsafe impl<T> Sync for FakeMutex<T> {}
