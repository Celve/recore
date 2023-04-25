use crate::{drivers::exit::QEMU_EXIT, time::get_time};

pub fn sys_shutdown(exit_code: usize) -> ! {
    QEMU_EXIT.exit(exit_code as u32);
}

pub fn sys_time() -> isize {
    get_time() as isize
}
