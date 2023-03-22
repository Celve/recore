use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use crate::config::{MEMORY_END, PAGE_SIZE};

use super::address::{PhyAddr, PhyPageNum};

/// The frame is the smallest granularity provided by frame allocator.
///
/// The creation of frame is equivalent to the allocation of frame allocator,
/// while the dropping of frame is equivalent to the deallocation of frame allocator.
pub struct Frame {
    ppn: PhyPageNum,
}

pub struct FrameAllocator {
    start: PhyPageNum,
    end: PhyPageNum,
    recycled: Vec<PhyPageNum>,
}

impl Frame {
    pub fn new() -> Self {
        let result = Self { ppn: alloc_frame() };
        result.init();
        result
    }

    pub fn from_existed(ppn: PhyPageNum) -> Self {
        Self { ppn }
    }

    pub fn init(&self) {
        let ptr = usize::from(self.ppn) as *mut u8;
        unsafe {
            core::slice::from_raw_parts_mut(ptr, PAGE_SIZE).fill(0);
        }
    }

    pub fn ppn(&self) -> PhyPageNum {
        self.ppn
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        dealloc_frame(self.ppn);
    }
}

impl FrameAllocator {
    pub fn empty() -> FrameAllocator {
        FrameAllocator {
            start: PhyPageNum(0),
            end: PhyPageNum(0),
            recycled: Vec::new(),
        }
    }

    pub fn new(start: usize, end: usize) -> FrameAllocator {
        FrameAllocator {
            start: PhyAddr::from(start).ceil_to_phy_page_num(),
            end: PhyAddr::from(end).floor_to_phy_page_num(),
            recycled: Vec::new(),
        }
    }

    pub fn init(&mut self, start: usize, end: usize) {
        if self.start != PhyPageNum(0) || self.end != PhyPageNum(0) {
            panic!("[frame_allocator] Frame allocator cannot be initialize twice.");
        }

        self.start = PhyAddr::from(start).ceil_to_phy_page_num();
        self.end = PhyAddr::from(end).floor_to_phy_page_num();
    }

    pub fn alloc(&mut self) -> Option<PhyPageNum> {
        let candidate = self.recycled.pop();
        if let Some(ppn) = candidate {
            Some(ppn)
        } else if self.start < self.end {
            let ppn = self.start;
            self.start += 1;
            Some(ppn)
        } else {
            None
        }
    }

    pub fn dealloc(&mut self, ppn: PhyPageNum) {
        self.recycled.push(ppn);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::empty());
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.lock().init(ekernel as usize, MEMORY_END);
}

fn alloc_frame() -> PhyPageNum {
    FRAME_ALLOCATOR
        .lock()
        .alloc()
        .expect("[frame_allocator] Cannot fetch any more frame.")
}

fn dealloc_frame(ppn: PhyPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
}
