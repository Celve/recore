#[repr(C)]
struct TrapContext {
    saved_regs: [usize; 32],
    user_sepc: usize,
    user_satp: usize,
    kernel_sp: usize,
    kernel_pc: usize, // addr of trap handler
    kernel_satp: usize,
}
