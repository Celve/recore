use alloc::sync::Arc;

use crate::{
    config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE_ADDR, USER_STACK_SIZE},
    mm::{
        address::VirAddr,
        area::Area,
        memory::MappingPermission,
        page_table::{PageTable, KERNEL_PAGE_TABLE},
    },
};

pub struct KernelStack {
    gid: usize,
    area: Area,
}

impl KernelStack {
    pub fn new(gid: usize) -> Self {
        let start_va = VirAddr::from(TRAMPOLINE_ADDR - KERNEL_STACK_SIZE * (gid + 1));
        let end_va = VirAddr::from(TRAMPOLINE_ADDR - KERNEL_STACK_SIZE * gid);
        let area = KERNEL_PAGE_TABLE.new_framed_area(
            start_va.floor_to_vir_page_num(),
            end_va.ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        );
        Self { gid, area }
    }

    pub fn top(&self) -> VirAddr {
        VirAddr::from(TRAMPOLINE_ADDR - KERNEL_STACK_SIZE * self.gid)
    }
}

pub struct UserStack {
    base: VirAddr,
    tid: usize,
    area: Area,
}

impl UserStack {
    pub fn new(base: VirAddr, tid: usize, page_table: &Arc<PageTable>) -> Self {
        Self {
            base,
            tid,
            area: page_table.new_framed_area(
                (base + (tid - 1) * (USER_STACK_SIZE + PAGE_SIZE)).floor_to_vir_page_num(),
                (base + tid * USER_STACK_SIZE + (tid - 1) * PAGE_SIZE).ceil_to_vir_page_num(),
                MappingPermission::R | MappingPermission::W | MappingPermission::U,
            ),
        }
    }

    pub fn renew(&self, page_table: &Arc<PageTable>) -> Self {
        Self {
            base: self.base,
            tid: self.tid,
            area: self.area.renew(page_table),
        }
    }

    pub fn top(&self) -> VirAddr {
        self.base + self.tid * USER_STACK_SIZE + (self.tid - 1) * PAGE_SIZE
    }
}
