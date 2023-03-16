use core::cmp::min;

use alloc::vec::Vec;

use crate::config::PAGE_SIZE;

use super::{
    address::{PhyPageNum, VirPageNum},
    frame::Frame,
    memory::{MappingPermission, MappingType},
    range::Range,
};

pub struct Area {
    range: Range<VirPageNum>,
    frames: Vec<Frame>,
    map_type: MappingType,
    map_perm: MappingPermission,
}

impl Area {
    /// Create a new area with identical mapping.
    ///
    /// `start` is included but `end` is not included.
    pub fn new_identical(start: VirPageNum, end: VirPageNum, map_perm: MappingPermission) -> Self {
        let range = Range::new(start, end);
        Self {
            range,
            frames: range
                .iter()
                .map(|vpn| Frame::from_existed(vpn.into()))
                .collect(),
            map_type: MappingType::Identical,
            map_perm,
        }
    }

    pub fn new_framed(start: VirPageNum, end: VirPageNum, map_perm: MappingPermission) -> Self {
        let range = Range::new(start, end);
        Self {
            range,
            frames: range.iter().map(|_| Frame::new()).collect(),
            map_type: MappingType::Framed,
            map_perm,
        }
    }

    pub fn new_linear(
        start_vpn: VirPageNum,
        start_ppn: PhyPageNum,
        len: usize,
        map_perm: MappingPermission,
    ) -> Self {
        Self {
            range: Range::new(start_vpn, start_vpn + len),
            frames: (0..len)
                .map(|offset| Frame::from_existed(start_ppn + offset))
                .collect(),
            map_type: MappingType::Linear,
            map_perm,
        }
    }

    pub fn copy_from(&self, data: &[u8]) {
        let mut start = 0;
        let len = data.len();
        for frame in self.frames.iter() {
            let src = &data[start..min(len, start + PAGE_SIZE)];
            let dst = frame.ppn().as_raw_bytes();
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
        }
    }

    pub fn init(&self) {
        self.frames.iter().for_each(|frame| {
            let ptr = usize::from(frame.ppn()) as *mut u8;
            unsafe {
                core::slice::from_raw_parts_mut(ptr, PAGE_SIZE).fill(0);
            }
        })
    }

    pub fn frame(&self, index: usize) -> &Frame {
        &self.frames[index]
    }

    pub fn map_perm(&self) -> MappingPermission {
        self.map_perm
    }

    pub fn range(&self) -> Range<VirPageNum> {
        self.range
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }
}
