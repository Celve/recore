use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

use spin::Spin;

use crate::{config::MEM_SEC_NUM, mm::address::PhyPageNum};

use super::Page;

pub struct MemUnit {
    pub start_ppn: PhyPageNum,
    pub end_ppn: PhyPageNum,
    pub ptr: &'static [MaybeUninit<Spin<Page>>],
}

pub struct MemTable {
    units: [MemUnit; MEM_SEC_NUM],
    len: usize,
}

pub static MEM_TABLE: Spin<MemTable> = Spin::new(MemTable::default());

impl MemTable {
    pub fn get(&self, ppn: PhyPageNum) -> Option<&Spin<Page>> {
        self.units
            .iter()
            .find(|unit| unit.start_ppn <= ppn && ppn < unit.end_ppn)
            .map(|unit| {
                let offset = ppn - unit.start_ppn;
                unsafe { unit.ptr[offset].assume_init_ref() }
            })
    }

    pub fn push(&mut self, unit: MemUnit) {
        self.units[self.len] = unit;
        self.len += 1;
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self {
            units: Default::default(),
            len: Default::default(),
        }
    }
}

impl MemUnit {
    pub fn new(
        start_ppn: PhyPageNum,
        end_ppn: PhyPageNum,
        ptr: &'static [MaybeUninit<Spin<Page>>],
    ) -> Self {
        Self {
            start_ppn,
            end_ppn,
            ptr,
        }
    }
}

impl Default for MemUnit {
    fn default() -> Self {
        Self {
            start_ppn: PhyPageNum(0),
            end_ppn: PhyPageNum(0),
            ptr: Default::default(),
        }
    }
}
