use core::arch::global_asm;

global_asm!(include_str!("trampoline.s"));

// global_asm!(
// ".altmacro",
// ".macro STORE_REG n",
// "    sd x\n, \n*8(sp)",
// ".endm",
// ".macro LOAD_REG n",
// "    ld x\n, \n*8(sp)",
// ".endm"
// );
