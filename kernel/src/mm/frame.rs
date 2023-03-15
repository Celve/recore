use core::cmp::min;

use super::{
    address::{PhyAddr, PhyPageNum, VirPageNum},
    memory::{MappingPermission, MappingType},
    page_table::PageTable,
};
use crate::{
    config::{MEMORY_END, PAGE_SIZE},
    sync::up::UpCell,
};
use alloc::vec::Vec;
use lazy_static::lazy_static;

/// The frame is the smallest granularity provided by frame allocator.
///
/// The creation of frame is equivalent to the allocation of frame allocator,
/// while the dropping of frame is equivalent to the deallocation of frame allocator.
pub struct Frame {
    ppn: PhyPageNum,
}

pub struct Area {
    ppns: Vec<PhyPageNum>,
    map_type: MappingType,
    map_perm: MappingPermission,
}

pub struct Iter<'a> {
    series: &'a Area,
    idx: usize,
}

pub struct FrameAllocator {
    start: PhyPageNum,
    end: PhyPageNum,
    recycled: Vec<PhyPageNum>,
}

impl Area {
    pub fn new_framed(len: usize, map_perm: MappingPermission) -> Self {
        let result = Self {
            ppns: (0..len).map(|_| alloc_frame()).collect(),
            map_type: MappingType::Framed,
            map_perm,
        };
        result.init();
        result
    }

    /// Create a new area with identical mapping.
    ///
    /// `start` is included while `end` is not excluded.
    pub fn new_identical(start: VirPageNum, end: VirPageNum, map_perm: MappingPermission) -> Self {
        let len = end - start;
        Self {
            ppns: (0..len).map(|i| PhyPageNum(start.0 + i)).collect(),
            map_type: MappingType::Identical,
            map_perm,
        }
    }

    pub fn copy_from(&self, data: &[u8]) {
        let mut start = 0;
        let len = data.len();
        for ppn in self.ppns.iter() {
            let src = &data[start..min(len, start + PAGE_SIZE)];
            let dst = ppn.as_raw_bytes();
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
        }
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

    pub fn map_perm(&self) -> MappingPermission {
        self.map_perm
    }

    pub fn iter(&self) -> Iter {
        Iter {
            series: self,
            idx: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.ppns.len()
    }
}

impl Drop for Area {
    fn drop(&mut self) {
        if self.map_type != MappingType::Identical {
            for ppn in self.ppns.iter() {
                dealloc_frame(*ppn);
            }
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = PhyPageNum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.series.ppns.len() {
            let result = self.series.ppns[self.idx];
            self.idx += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl Frame {
    pub fn new() -> Self {
        let result = Self { ppn: alloc_frame() };
        result.init();
        result
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

fn alloc_frame() -> PhyPageNum {
    FRAME_ALLOCATOR
        .borrow_mut()
        .alloc()
        .expect("[frame_allocator] Cannot fetch any more frame.")
}

fn dealloc_frame(ppn: PhyPageNum) {
    FRAME_ALLOCATOR.borrow_mut().dealloc(ppn);
}
