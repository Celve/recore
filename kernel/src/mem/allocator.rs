use crate::mm::address::PhyPageNum;

pub trait PageAllocator {
    unsafe fn init(&self, start: PhyPageNum, end: PhyPageNum);
    fn alloc_page(&self) -> PhyPageNum;
    fn dealloc_page(&self, ppn: PhyPageNum);
}
