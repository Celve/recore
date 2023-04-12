use super::{area::Area, page_table::PageTable};

use crate::mm::page_table::KERNEL_PAGE_TABLE;

use alloc::{sync::Arc, vec::Vec};
use bitflags::bitflags;
use lazy_static::lazy_static;

pub struct MemSet {
    areas: Vec<Area>,
}

bitflags! {
    pub struct MappingPermission: u8 {
        const R = 1 << 1; // Bit used to indicate readability.
        const W = 1 << 2; // Bit used to indicate writability.
        const X = 1 << 3; // Bit used to indicate whether executable.
        const U = 1 << 4; // Bit used to indicate user's accessibility.
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MappingType {
    Identical,
    Framed,
    Linear,
}

impl MemSet {
    pub fn new(areas: Vec<Area>) -> Self {
        Self { areas }
    }

    pub fn len(&self) -> usize {
        self.areas.len()
    }

    pub fn renew(&self, page_table: &Arc<PageTable>) -> Self {
        Self {
            areas: self
                .areas
                .iter()
                .map(|area| area.renew(page_table))
                .collect(),
        }
    }
}

lazy_static! {
    pub static ref KERNEL_MEMSET: MemSet = KERNEL_PAGE_TABLE.new_kernel();
}
