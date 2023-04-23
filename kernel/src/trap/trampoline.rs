use core::arch::{asm, global_asm};

use crate::config::{MIN_AVG_TIME_SLICE, MIN_EXEC_TIME_SLICE, TRAMPOLINE_ADDR};
use crate::task::processor::{fetch_curr_proc, fetch_curr_task};
use crate::time::get_time;
use crate::trap::set_user_stvec;

global_asm!(include_str!("trampoline.s"));

fn should_yield() -> bool {
    let task = fetch_curr_task();
    let task_guard = task.lock();
    let task_time = task_guard.task_time();
    if task_time.remaining() < MIN_EXEC_TIME_SLICE {
        // if the remaining time is less or there is a blocked task wake up
        true
    } else {
        false
    }
}

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

    // yield
    if should_yield() {
        if fetch_curr_proc().pid() == 8 {
            println!("yield");
        }
        fetch_curr_task().yield_now();
    }

    set_user_stvec();

    // set the timer
    fetch_curr_task().lock().task_time_mut().restore();

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
