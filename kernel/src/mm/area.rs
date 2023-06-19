use core::cmp::min;

use alloc::{sync::Arc, vec::Vec};

use crate::{config::PAGE_SIZE, mem::normal::page::NormalPageHandle};

use super::{
    address::{PhyPageNum, VirPageNum},
    memory::{MappingPermission, MappingType},
    page_table::PageTable,
    range::Range,
};

pub struct Area {
    range: Range<VirPageNum>,
    frames: Vec<NormalPageHandle>,
    map_type: MappingType,
    map_perm: MappingPermission,
    page_table: Arc<PageTable>,
}

impl Area {
    /// Create a new area with identical mapping.
    ///
    /// `start` is included but `end` is not included.
    pub fn new_identical(
        start: VirPageNum,
        end: VirPageNum,
        map_perm: MappingPermission,
        page_table: &Arc<PageTable>,
    ) -> Self {
        let range = Range::new(start, end);
        let res = Self {
            range,
            // frames: range
            //     .iter()
            //     .map(|vpn| NormalPageHandle::from_unalloc(vpn.into()))
            //     .collect(),
            frames: Default::default(),
            map_type: MappingType::Identical,
            map_perm,
            page_table: page_table.clone(),
        };
        // page_table.map_area(&res);
        range
            .iter()
            .for_each(|vpn| page_table.map(vpn.into(), vpn.into(), map_perm.into()));
        res
    }

    pub fn new_framed(
        start: VirPageNum,
        end: VirPageNum,
        map_perm: MappingPermission,
        page_table: &Arc<PageTable>,
    ) -> Self {
        let range = Range::new(start, end);
        let res = Self {
            range,
            frames: range.iter().map(|_| NormalPageHandle::new()).collect(),
            map_type: MappingType::Framed,
            map_perm,
            page_table: page_table.clone(),
        };
        page_table.map_area(&res);
        res
    }

    pub fn new_linear(
        start_vpn: VirPageNum,
        start_ppn: PhyPageNum,
        len: usize,
        map_perm: MappingPermission,
        page_table: &Arc<PageTable>,
    ) -> Self {
        // let res = Self {
        //     range: Range::new(start_vpn, start_vpn + len),
        //     frames: (0..len)
        //         .map(|offset| NormalPageHandle::from_unalloc(start_ppn + offset))
        //         .collect(),
        //     map_type: MappingType::Linear,
        //     map_perm,
        //     page_table: page_table.clone(),
        // };
        // page_table.map_area(&res);
        // res
        todo!()
    }

    pub fn renew(&self, page_table: &Arc<PageTable>) -> Self {
        match self.map_type {
            MappingType::Identical => {
                Self::new_identical(self.range.start, self.range.end, self.map_perm, page_table)
            }
            MappingType::Framed => {
                let res =
                    Self::new_framed(self.range.start, self.range.end, self.map_perm, page_table);
                res.copy_from_existed(self);
                res
            }
            MappingType::Linear => Self::new_linear(
                self.range.start,
                self.frames[0].ppn,
                self.len(),
                self.map_perm,
                page_table,
            ),
        }
    }

    pub fn copy_from_raw_bytes(&self, data: &[u8]) {
        let mut start = 0;
        let len = data.len();
        for frame in self.frames.iter() {
            let src = &data[start..min(len, start + PAGE_SIZE)];
            let dst = unsafe { &mut frame.ppn.as_raw_bytes()[..src.len()] };
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
        }
    }

    pub fn copy_from_existed(&self, other: &Area) {
        assert_eq!(self.len(), other.len());
        self.frames
            .iter()
            .zip(other.frames.iter())
            .for_each(|(dst, src)| unsafe {
                let dst_addr = dst.ppn.as_raw_bytes();
                let src_addr = src.ppn.as_raw_bytes();
                dst_addr.copy_from_slice(src_addr);
            });
    }

    pub fn init(&self) {
        self.frames.iter().for_each(|frame| {
            let ptr = usize::from(frame.ppn) as *mut u8;
            unsafe {
                core::slice::from_raw_parts_mut(ptr, PAGE_SIZE).fill(0);
            }
        })
    }
}

impl Drop for Area {
    fn drop(&mut self) {
        // self.page_table.unmap_area(self);
    }
}

impl Area {
    pub fn frames(&self) -> &Vec<NormalPageHandle> {
        &self.frames
    }

    pub fn frame(&self, index: usize) -> &NormalPageHandle {
        &self.frames[index]
    }

    pub fn map_perm(&self) -> MappingPermission {
        self.map_perm
    }

    pub fn map_type(&self) -> MappingType {
        self.map_type
    }

    pub fn range(&self) -> Range<VirPageNum> {
        self.range
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }
}
