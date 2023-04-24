mod cache;
mod locked_heap;
mod page;
pub mod slab_allocator;

use crate::config::{KERNEL_HEAP_SIZE, PAGE_SIZE, PAGE_SIZE_BITS};
use page::Page;

use self::{locked_heap::LockedHeap, page::PagePtr};

#[link_section = ".data.heap"]
static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
static mut MEM_MAP: [Page; KERNEL_HEAP_SIZE / PAGE_SIZE] =
    [Page::empty(); KERNEL_HEAP_SIZE / PAGE_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();
// static SLAB_ALLOCATOR: LockedSlabHeap = LockedSlabHeap::empty();

// static BUDDY_ALLOCATOR: LockedBuddyHeap = LockedBuddyHeap::empty(KERNEL_HEAP_GRANULARITY);

pub fn init_heap() {
    unsafe {
        let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
        let end = start + KERNEL_HEAP_SPACE.len();

        MEM_MAP.iter_mut().enumerate().for_each(|(i, page)| {
            *page.pa_mut() = i * PAGE_SIZE + start;
        });

        HEAP.init(start, end);
    }
}

pub fn fetch_page(ptr: usize) -> Option<PagePtr> {
    unsafe {
        let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
        Some(PagePtr::new(&mut MEM_MAP[(ptr - start) >> PAGE_SIZE_BITS]))
    }
}
