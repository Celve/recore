use crate::mm::address::PhyPageNum;

use super::allocator::PageAllocator;

/// This struct must be allocated statically. It's the corner stone of whole memory management.
///
/// In this stage, the allocation of memory deals nothing with the memory representation of `Page`.
/// What `Page` would be is determined in the next stage.
/// It's also possible that there are different `Page` inside one `MemSection`.
pub struct MemSec<T: PageAllocator> {
    pub allocator: T,
    pub start_ppn: PhyPageNum,
    pub end_ppn: PhyPageNum,
}

impl<T: PageAllocator> MemSec<T> {
    pub const fn empty(allocator: T) -> Self {
        Self {
            allocator,
            start_ppn: PhyPageNum(0), // uninit
            end_ppn: PhyPageNum(0),
        }
    }

    pub fn init(&mut self, start_ppn: PhyPageNum, end_ppn: PhyPageNum) {
        self.start_ppn = start_ppn;
        unsafe {
            self.allocator.init(start_ppn, end_ppn);
        }
    }

    pub fn alloc(&self) -> PhyPageNum {
        self.allocator.alloc_page()
    }

    pub fn dealloc(&self, ppn: PhyPageNum) {
        self.allocator.dealloc_page(ppn)
    }
}
