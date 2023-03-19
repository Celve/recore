use crate::config::{KERNEL_HEAP_GRANULARITY, KERNEL_HEAP_SIZE};
use crate::sync::up::UpCell;
use core::alloc::GlobalAlloc;
use core::ops::Deref;

use heap::buddy_allocator::BuddyAllocator;

static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub struct UpBuddyAllocator {
    inner: UpCell<BuddyAllocator>,
}

impl UpBuddyAllocator {
    pub const fn empty(gran: usize) -> Self {
        Self {
            inner: UpCell::new(BuddyAllocator::empty(gran)),
        }
    }

    pub fn init(&self) {
        unsafe {
            let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
            let end = start + KERNEL_HEAP_SPACE.len();
            self.borrow_mut().add_segment(start, end);
        }
    }
}

impl Deref for UpBuddyAllocator {
    type Target = UpCell<BuddyAllocator>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

unsafe impl GlobalAlloc for UpBuddyAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.inner.borrow_mut().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.inner.borrow_mut().dealloc(ptr, layout);
    }
}

#[global_allocator]
static HEAP_ALLOCATOR: UpBuddyAllocator = UpBuddyAllocator::empty(KERNEL_HEAP_GRANULARITY);

pub fn init_heap() {
    HEAP_ALLOCATOR.init();
}
