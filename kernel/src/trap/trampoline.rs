use core::arch::{asm, global_asm};

use crate::config::{PAGE_SIZE, TRAMPOLINE_ADDR};
use crate::task::processor::fetch_curr_task;
use crate::trap::set_user_stvec;

global_asm!(include_str!("trampoline.s"));

#[no_mangle]
pub fn restore() -> ! {
    // RECEIVE wrong SP when get into this function!
    let user_satp = fetch_curr_task().lock().page_table().to_satp();

    extern "C" {
        fn _restore();
        fn _alltraps();
    }

    set_user_stvec();

    unsafe {
        asm! {
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) TRAMPOLINE_ADDR + (_restore as usize - _alltraps as usize),
            in("a0") TRAMPOLINE_ADDR - PAGE_SIZE,
            in("a1") user_satp,
            options(noreturn),
        }
    }
}
