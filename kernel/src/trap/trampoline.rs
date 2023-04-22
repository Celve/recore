use core::arch::{asm, global_asm};

use riscv::register::sip;

use crate::config::TRAMPOLINE_ADDR;
use crate::task::processor::fetch_curr_task;
use crate::trap::set_user_stvec;

global_asm!(include_str!("trampoline.s"));

/// The function is a trampoline for `_restore()` inside `trampoline.s`.
/// It would never return when the function is called.
/// Hence, all stack frames inside the kernel stack is useless from this point.
#[no_mangle]
pub fn restore() -> ! {
    let user_satp = fetch_curr_task().lock().page_table().to_satp();
    let trap_ctx_ptr = fetch_curr_task().lock().trap_ctx_ptr();

    extern "C" {
        fn _restore();
        fn _alltraps();
    }

    // acknowledge the software interrupt again, because the supervisor might run too long
    // let sip = sip::read().bits();
    // unsafe {
    // asm! {"csrw sip, {sip}", sip = in(reg) sip ^ 2};
    // }

    set_user_stvec();

    unsafe {
        asm! {
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) TRAMPOLINE_ADDR + (_restore as usize - _alltraps as usize),
            in("a0") trap_ctx_ptr,
            in("a1") user_satp,
            options(noreturn),
        }
    }
}
