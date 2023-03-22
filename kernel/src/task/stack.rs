use crate::{
    config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE_START_ADDRESS},
    mm::{
        address::VirAddr,
        area::Area,
        memory::{MappingPermission, KERNEL_SPACE},
    },
};

pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    pub fn new(pid: usize) -> Self {
        let start_va = VirAddr::from(TRAMPOLINE_START_ADDRESS - KERNEL_STACK_SIZE * (pid + 1));
        let end_va = VirAddr::from(TRAMPOLINE_START_ADDRESS - KERNEL_STACK_SIZE * pid);
        KERNEL_SPACE.borrow_mut().map(Area::new_framed(
            start_va.floor_to_vir_page_num(),
            end_va.ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));
        Self { pid }
    }

    pub fn top(&self) -> VirAddr {
        VirAddr::from(TRAMPOLINE_START_ADDRESS - KERNEL_STACK_SIZE * self.pid)
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let start_va = VirAddr::from(TRAMPOLINE_START_ADDRESS - KERNEL_STACK_SIZE * (self.pid + 1));
        KERNEL_SPACE
            .borrow_mut()
            .unmap(start_va.floor_to_vir_page_num());
    }
}
