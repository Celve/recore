use alloc::vec::Vec;
use spin::Spin;

use crate::{mem::allocator::PageAllocator, mm::address::PhyPageNum};

pub struct FrameAllocator {
    inner: Spin<FrameAllocatorInner>,
}

/// The allocator for page table.
pub struct FrameAllocatorInner {
    start: PhyPageNum,
    end: PhyPageNum,
    recycled: Vec<PhyPageNum>,
}

impl const Default for FrameAllocatorInner {
    fn default() -> Self {
        Self {
            start: PhyPageNum(0),
            end: PhyPageNum(0),
            recycled: Default::default(),
        }
    }
}

impl const Default for FrameAllocator {
    fn default() -> Self {
        Self {
            inner: Spin::new(FrameAllocatorInner::default()),
        }
    }
}

impl PageAllocator for FrameAllocator {
    /// The init could be done only once.
    unsafe fn init(&self, start: PhyPageNum, end: PhyPageNum) {
        let mut guard = self.inner.lock();
        guard.start = start;
        guard.end = end;
        guard.recycled.clear();
    }

    fn alloc_page(&self) -> PhyPageNum {
        let mut guard = self.inner.lock();
        let candidate = guard.recycled.pop();
        if let Some(ppn) = candidate {
            ppn
        } else if guard.start < guard.end {
            let ppn = guard.start;
            guard.start += 1;
            ppn
        } else {
            PhyPageNum::null()
        }
    }

    fn dealloc_page(&self, ppn: PhyPageNum) {
        self.inner.lock().recycled.push(ppn);
    }
}
