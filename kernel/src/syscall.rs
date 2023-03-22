use crate::{
    io::stdout::Stdout,
    mm::page_table::translate_bytes,
    println,
    task::{exit_and_yield, manager::fetch_curr_task},
};

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

pub fn syscall(id: usize, args: [usize; 3]) {
    match id {
        SYSCALL_READ => todo!(),
        SYSCALL_WRITE => syscall_write(args[0], args[1], args[2]),
        SYSCALL_EXIT => syscall_exit(args[0] as isize),
        _ => todo!(),
    }
}

pub fn syscall_write(fd: usize, buffer_ptr: usize, buffer_len: usize) {
    if fd != 1 {
        panic!("[syscall] Doesn't support file write.");
    }
    let task_cell = fetch_curr_task();
    let task = task_cell.borrow_mut();
    let page_table = task.user_mem().page_table();
    let buffer = translate_bytes(page_table, buffer_ptr as *const u8, buffer_len);
    let stdout = Stdout;
    buffer.iter().for_each(|b| stdout.putchar(*b as char));
}

pub fn syscall_exit(exit_code: isize) {
    exit_and_yield(exit_code);
}
