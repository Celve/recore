use core::arch::{asm, global_asm};

use crate::config::{MIN_EXEC_TIME_SLICE, TRAMPOLINE_ADDR};
use crate::task::processor::Processor;
use crate::trap::set_user_stvec;

global_asm!(include_str!("trampoline.s"));

fn should_yield() -> bool {
    let task = Processor::curr_task();
    let task_guard = task.lock();
    let task_time = &task_guard.task_time;
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
    let user_satp = Processor::curr_task().lock().page_table().to_satp();
    let trap_ctx_ptr = Processor::curr_task().lock().trap_ctx_ptr();

    extern "C" {
        fn _restore();
        fn _alltraps();
    }

    // yield
    if should_yield() {
        Processor::yield_now();
    }

    set_user_stvec();

    // set the timer
    Processor::curr_task().lock().task_time.restore();

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
