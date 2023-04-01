use core::arch::{asm, global_asm};

use riscv::register::utvec::TrapMode;

use crate::config::{TRAMPOLINE_START_ADDRESS, TRAP_CONTEXT_START_ADDRESS};
use crate::mm::address::VirPageNum;
use crate::task::processor::fetch_curr_task;
use crate::trap::set_user_stvec;

global_asm!(include_str!("trampoline.s"));

#[no_mangle]
pub fn restore() -> ! {
    fetch_curr_task()
        .lock()
        .user_mem()
        .page_table()
        .translate_vpn(VirPageNum::from(0x10000));
    let user_satp = fetch_curr_task().lock().user_mem().page_table().to_satp();

    extern "C" {
        fn _restore();
        fn _alltraps();
    }

    set_user_stvec();

    unsafe {
        asm! {
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) TRAMPOLINE_START_ADDRESS + (_restore as usize - _alltraps as usize),
            in("a0") TRAP_CONTEXT_START_ADDRESS,
            in("a1") user_satp,
            options(noreturn),
        }
    }
}
