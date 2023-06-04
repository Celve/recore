use core::{alloc::GlobalAlloc, cmp::max, mem::size_of};

use spin::Spin;

use crate::config::{KERNEL_HEAP_GRANULARITY, KERNEL_HEAP_SIZE, PAGE_SIZE_BITS};

use self::{allocator::BuddyAllocator, cache::Cache};

use super::section::MemSec;

pub mod allocator;
pub mod cache;
pub mod linked_list;
pub mod page;

#[link_section = ".data.heap"]
static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

static mut SLAB_MEM_SECTION: MemSec<BuddyAllocator> =
    MemSec::empty(BuddyAllocator::empty(KERNEL_HEAP_GRANULARITY));

#[global_allocator]
static KERNEL_HEAP: SlabAllocator = SlabAllocator::empty();

pub fn init_slab() {
    unsafe {
        let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
        let end = start + KERNEL_HEAP_SIZE;
        SLAB_MEM_SECTION.init(start.into(), end.into());
        KERNEL_HEAP.init();
    }
}

pub struct SlabAllocator {
    /// Different order allocate different size, which ranges from 1 byte to 4096 bytes.  
    caches: [Spin<Cache>; PAGE_SIZE_BITS + 1],
}

impl SlabAllocator {
    pub const fn empty() -> Self {
        let caches = [const { Spin::new(Cache::empty()) }; PAGE_SIZE_BITS + 1];
        Self { caches }
    }

    pub fn init(&self) {
        self.caches.iter().enumerate().for_each(|(i, cache)| {
            *cache.lock().order_mut() = i;
        });
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = max(size_of::<usize>(), layout.size().next_power_of_two());
        let order = size.trailing_zeros() as usize;
        if order <= PAGE_SIZE_BITS {
            self.caches[order].lock().alloc() as *mut u8
        } else {
            SLAB_MEM_SECTION.allocator.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let size = max(size_of::<usize>(), layout.size().next_power_of_two());
        let order = size.trailing_zeros() as usize;
        if order <= PAGE_SIZE_BITS {
            self.caches[order].lock().dealloc(ptr as usize);
        } else {
            SLAB_MEM_SECTION.allocator.dealloc(ptr, layout);
        }
    }
}
