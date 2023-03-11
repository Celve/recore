use crate::sync::up::UpCell;
use alloc::{sync::Arc, vec::Vec};

use super::{
    address::{PhyPageNum, VirPageNum},
    frame_allocator::{alloc_series, Series},
    page_table::{PTEFlags, PageTable},
    range::Range,
};

pub struct MemorySet {
    page_table: Arc<UpCell<PageTable>>,
    mapping_areas: Vec<MappingArea>,
}

pub struct MappingArea {
    range: Range<VirPageNum>,
    page_table: Arc<UpCell<PageTable>>,
    series: Arc<UpCell<Series>>,
    mapping_type: MappingType,
    mapping_flags: PTEFlags,
}

pub enum MappingType {
    Identical,
    Framed,
    Linear,
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: Arc::new(UpCell::new(PageTable::new())),
            mapping_areas: Vec::new(),
        }
    }

    pub fn push(
        &mut self,
        start: VirPageNum,
        end: VirPageNum,
        mapping_type: MappingType,
        mapping_flags: PTEFlags,
    ) {
        self.mapping_areas.push(MappingArea::new(
            start,
            end,
            Arc::clone(&self.page_table),
            mapping_type,
            mapping_flags,
        ));
    }

    pub fn remove(&mut self, start: VirPageNum) -> bool {
        let pos = self
            .mapping_areas
            .iter()
            .position(|area| area.range.start == start);

        match pos {
            Some(pos) => {
                self.mapping_areas.remove(pos);
                true
            }
            None => false,
        }
    }

    pub fn page_table(&self) -> &Arc<UpCell<PageTable>> {
        &self.page_table
    }
}

impl MappingArea {
    pub fn new(
        start: VirPageNum,
        end: VirPageNum,
        page_table: Arc<UpCell<PageTable>>,
        mapping_type: MappingType,
        mapping_flags: PTEFlags,
    ) -> Self {
        let range = Range::new(start, end);
        let mut result = Self {
            range,
            page_table,
            series: Arc::new(UpCell::new(match mapping_type {
                MappingType::Identical => Series::new(
                    range.iter().map(|vpn| PhyPageNum::from(vpn)).collect(),
                    false,
                ),
                MappingType::Framed => alloc_series(end - start),
                MappingType::Linear => todo!(),
            })),
            mapping_type,
            mapping_flags,
        };
        result.map();
        result
    }

    pub fn map(&mut self) {
        let mut page_table = self.page_table.borrow_mut();
        let series = self.series.borrow_mut();
        self.range
            .iter()
            .enumerate()
            .for_each(|(i, vpn)| page_table.map(vpn, series.ppn(i), self.mapping_flags));
    }

    pub fn unmap(&mut self) {
        let mut page_table = self.page_table.borrow_mut();
        self.range.iter().for_each(|vpn| page_table.unmap(vpn));
    }
}

impl Drop for MappingArea {
    fn drop(&mut self) {
        self.unmap();
    }
}
