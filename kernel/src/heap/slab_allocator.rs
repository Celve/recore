use core::{
    alloc::{GlobalAlloc, Layout},
    cmp::max,
    mem::size_of,
};

use alloc::vec::Vec;
use spin::mutex::Mutex;

use crate::config::PAGE_SIZE_BITS;

use super::cache::Cache;

pub struct SlabAllocator {
    /// Different order allocate different size, which ranges from 1 byte to 4096 bytes.  
    caches: [Cache; PAGE_SIZE_BITS + 1],
}

pub struct LockedSlabHeap {
    allocator: Mutex<SlabAllocator>,
}

impl SlabAllocator {
    pub const fn empty() -> Self {
        let caches = [Cache::empty(); PAGE_SIZE_BITS + 1];
        Self { caches }
    }

    pub fn init(&mut self) {
        self.caches.iter_mut().enumerate().for_each(|(i, cache)| {
            *cache.order_mut() = i;
        });
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = max(size_of::<usize>(), layout.size().next_power_of_two());
        let order = size.trailing_zeros() as usize;
        assert!(order <= PAGE_SIZE_BITS);
        self.caches[order].alloc() as *mut u8
    }

    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = max(size_of::<usize>(), layout.size().next_power_of_two());
        let order = size.trailing_zeros() as usize;
        assert!(order <= PAGE_SIZE_BITS);
        self.caches[order].dealloc(ptr as usize);
    }
}

impl LockedSlabHeap {
    pub const fn empty() -> Self {
        Self {
            allocator: Mutex::new(SlabAllocator::empty()),
        }
    }

    pub fn init(&self) {
        self.allocator.lock().init();
    }
}

unsafe impl GlobalAlloc for LockedSlabHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.allocator.lock().dealloc(ptr, layout);
    }
}

pub fn test_slab_allocator() {
    let mut v = Vec::new();
    (0..100).for_each(|i| v.push(i));
    (0..100).for_each(|i| println!("{}", v.pop().unwrap()));
}
