use core::arch::{asm, global_asm};

global_asm!(
    ".macro STORE_REG n",
    "sd x\\n, \\n*8(sp)",
    ".endm",
    ".macro LOAD_REG n",
    "ld x\\n, \\n*8(sp)",
    ".endm "
);

#[naked]
pub unsafe extern "C" fn trampoline() {
    asm!(".set n, 1", "STORE_REG n", options(noreturn));
}
