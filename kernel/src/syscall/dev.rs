use crate::drivers::exit::QEMU_EXIT;

pub fn sys_shutdown(exit_code: usize) -> ! {
    QEMU_EXIT.exit(exit_code as u32);
}
