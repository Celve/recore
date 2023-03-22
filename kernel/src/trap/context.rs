use riscv::register::mcause::Trap;

use crate::println;

use super::trap_handler;

#[repr(C)]
pub struct TrapContext {
    pub saved_regs: [usize; 32],
    pub user_sepc: usize,
    pub user_sstatus: usize,
    pub kernel_sp: usize,
    pub kernel_pc: usize, // addr of trap handler
    pub kernel_satp: usize,
}

impl TrapContext {
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
        println!("[trap]: user's sp: {:#x}", user_sp);
        Self {
            saved_regs,
            user_sepc,
            user_sstatus,
            kernel_sp,
            kernel_pc,
            kernel_satp,
        }
    }
}
