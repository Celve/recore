use alloc::sync::Arc;

use crate::{
    config::{PAGE_SIZE, TRAMPOLINE_ADDR},
    mm::{address::VirAddr, area::Area, memory::MappingPermission, page_table::PageTable},
};

#[repr(C)]
#[derive(Clone)]
pub struct TrapCtx {
    pub saved_regs: [usize; 32],
    pub user_sepc: usize,
    pub user_sstatus: usize,
    pub kernel_sp: usize,
    pub kernel_pc: usize, // addr of trap handler
    pub kernel_satp: usize,
}

pub struct TrapCtxHandle {
    tid: usize,
    area: Area,
}

impl TrapCtx {
    pub fn new(
        user_sp: usize,
        user_sepc: usize,
        user_sstatus: usize,
        kernel_sp: usize,
        kernel_pc: usize,
        kernel_satp: usize,
    ) -> Self {
        let mut saved_regs: [usize; 32] = [0; 32];
        saved_regs[2] = user_sp;
        println!("[trap] User's sp: {:#x}", user_sp);
        Self {
            saved_regs,
            user_sepc,
            user_sstatus,
            kernel_sp,
            kernel_pc,
            kernel_satp,
        }
    }

    pub fn a0(&self) -> usize {
        self.saved_regs[10]
    }

    pub fn a0_mut(&mut self) -> &mut usize {
        &mut self.saved_regs[10]
    }

    pub fn a1_mut(&mut self) -> &mut usize {
        &mut self.saved_regs[11]
    }

    pub fn user_sp(&self) -> usize {
        self.saved_regs[2]
    }

    pub fn user_sp_mut(&mut self) -> &mut usize {
        &mut self.saved_regs[2]
    }
}

impl TrapCtxHandle {
    pub fn new(tid: usize, page_table: &Arc<PageTable>) -> Self {
        println!(
            "{:#x} {:#x}",
            TRAMPOLINE_ADDR - PAGE_SIZE * tid,
            TRAMPOLINE_ADDR - PAGE_SIZE * (tid - 1)
        );
        Self {
            tid,
            area: page_table.new_framed_area(
                VirAddr::from(TRAMPOLINE_ADDR - PAGE_SIZE * tid).floor_to_vir_page_num(),
                VirAddr::from(TRAMPOLINE_ADDR - PAGE_SIZE * (tid - 1)).ceil_to_vir_page_num(),
                MappingPermission::R | MappingPermission::W,
            ),
        }
    }

    pub fn renew(&self, page_table: &Arc<PageTable>) -> Self {
        Self {
            tid: self.tid,
            area: self.area.renew(page_table),
        }
    }

    pub fn trap_ctx(&self) -> &TrapCtx {
        unsafe { &*(usize::from(self.area.frames().first().unwrap().ppn()) as *const TrapCtx) }
    }

    pub fn trap_ctx_mut(&self) -> &mut TrapCtx {
        unsafe { &mut *(usize::from(self.area.frames().first().unwrap().ppn()) as *mut TrapCtx) }
    }

    pub fn trap_ctx_ptr(&self) -> usize {
        TRAMPOLINE_ADDR - PAGE_SIZE * self.tid
    }
}
