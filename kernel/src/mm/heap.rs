use allocator::heap::LockedHeap;

use crate::config::{KERNEL_HEAP_GRANULARITY, KERNEL_HEAP_SIZE};

static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty(KERNEL_HEAP_GRANULARITY);

pub fn init_heap() {
    unsafe {
        let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
        let end = start + KERNEL_HEAP_SPACE.len();
        HEAP_ALLOCATOR.add_segment(start, end);
    }
}
