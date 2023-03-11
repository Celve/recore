use super::address::{PhyAddr, PhyPageNum};
use crate::{
    config::{MEMORY_END, PAGE_SIZE},
    sync::up::UpCell,
};
use alloc::vec::Vec;
use lazy_static::lazy_static;

/// The frame is the smallest granularity provided by frame allocator.
/// And it's the main approach to dealing with allocation and deallocation.  
///
/// The creation of frame is equivalent to the allocation of frame allocator,
/// while the dropping of frame is equivalent to the deallocation of frame allocator.
pub struct Frame {
    ppn: PhyPageNum,
}

pub struct Series {
    ppns: Vec<PhyPageNum>,
    is_allocated: bool,
}

pub struct FrameAllocator {
    start: PhyPageNum,
    end: PhyPageNum,
    recycled: Vec<PhyPageNum>,
}

impl Series {
    pub fn new(ppns: Vec<PhyPageNum>, is_allocated: bool) -> Self {
        Self { ppns, is_allocated }
    }

    pub fn init(&self) {
        self.ppns.iter().for_each(|ppn| {
            let ptr = usize::from(*ppn) as *mut u8;
            unsafe {
                core::slice::from_raw_parts_mut(ptr, PAGE_SIZE).fill(0);
            }
        })
    }

    pub fn ppn(&self, index: usize) -> PhyPageNum {
        self.ppns[index]
    }
}

impl Drop for Series {
    fn drop(&mut self) {
        if self.is_allocated {
            dealloc_series(self);
        }
    }
}

impl Frame {
    pub fn new(ppn: PhyPageNum) -> Self {
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
        dealloc_frame(self);
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
    pub static ref FRAME_ALLOCATOR: UpCell<FrameAllocator> = UpCell::new(FrameAllocator::empty());
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR
        .borrow_mut()
        .init(ekernel as usize, MEMORY_END);
}

pub fn alloc_frame() -> Frame {
    let result = Frame::new(
        FRAME_ALLOCATOR
            .borrow_mut()
            .alloc()
            .expect("[frame_allocator] Cannot fetch any more frame."),
    );
    result.init();
    result
}

pub fn dealloc_frame(frame: &Frame) {
    FRAME_ALLOCATOR.borrow_mut().dealloc(frame.ppn());
}

/// Allocate a set of frames with the given number.
///
/// `num` means the number of frames, instead of the number of bytes or something else.
pub fn alloc_series(num: usize) -> Series {
    let mut frame_allocator = FRAME_ALLOCATOR.borrow_mut();
    let mut ppns = Vec::new();
    for _ in 0..num {
        ppns.push(frame_allocator.alloc().unwrap());
    }
    let result = Series::new(ppns, true);
    result.init();
    result
}

pub fn dealloc_series(series: &Series) {
    let mut frame_allocator = FRAME_ALLOCATOR.borrow_mut();
    for ppn in series.ppns.iter() {
        frame_allocator.dealloc(*ppn);
    }
}
