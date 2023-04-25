use core::{
    alloc::{GlobalAlloc, Layout},
    cmp::max,
    mem::size_of,
};

use alloc::vec::Vec;
use spin::Spin;

use crate::config::PAGE_SIZE_BITS;

use super::cache::Cache;

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
        assert!(order <= PAGE_SIZE_BITS);
        self.caches[order].lock().alloc() as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let size = max(size_of::<usize>(), layout.size().next_power_of_two());
        let order = size.trailing_zeros() as usize;
        assert!(order <= PAGE_SIZE_BITS);
        self.caches[order].lock().dealloc(ptr as usize);
    }
}
