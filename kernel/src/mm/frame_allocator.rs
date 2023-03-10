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
pub struct FrameSet {
    frames: Vec<Frame>,
}

pub struct Frame {
    ppn: PhyPageNum,
    is_allocated: bool,
}

pub struct FrameAllocator {
    start: PhyPageNum,
    end: PhyPageNum,
    recycled: Vec<PhyPageNum>,
}

impl FrameSet {
    pub fn new(frames: Vec<Frame>) -> Self {
        Self { frames }
    }

    pub fn frames(&self, index: usize) -> &Frame {
        &self.frames[index]
    }
}

impl Frame {
    pub fn new(ppn: PhyPageNum, is_allocated: bool) -> Self {
        Self { ppn, is_allocated }
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

    pub fn alloc(&mut self) -> Option<Frame> {
        let candidate = self.recycled.pop();
        if let Some(ppn) = candidate {
            Some(Frame::new(ppn, true))
        } else if self.start < self.end {
            let ppn = self.start;
            let test = usize::from(PhyAddr::from(ppn));
            self.start += 1;
            Some(Frame::new(ppn, true))
        } else {
            None
        }
    }

    pub fn dealloc(&mut self, frame: &Frame) {
        self.recycled.push(frame.ppn());
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
#[no_mangle]
pub fn alloc_frame() -> Frame {
    let result = FRAME_ALLOCATOR
        .borrow_mut()
        .alloc()
        .expect("[frame_allocator] Cannot fetch any more frame.");
    result.init();
    result
}

pub fn dealloc_frame(frame: &Frame) {
    FRAME_ALLOCATOR.borrow_mut().dealloc(frame);
}

/// Allocate a set of frames with the given number.
///
/// `num` means the number of frames, instead of the number of bytes or something else.
pub fn alloc_frame_set(num: usize) -> FrameSet {
    let mut frames = Vec::new();
    for _ in 0..num {
        frames.push(alloc_frame());
    }
    FrameSet::new(frames)
}

pub fn dealloc_frame_set(frames: FrameSet) {
    for frame in frames.frames.iter() {
        dealloc_frame(frame);
    }
}
