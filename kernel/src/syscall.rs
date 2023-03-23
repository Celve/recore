use crate::{
    io::{stdin::Stdin, stdout::Stdout},
    mm::page_table::translate_bytes,
    task::{exit_and_yield, processor::fetch_curr_task, suspend_and_yield},
};

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;

pub fn syscall(id: usize, args: [usize; 3]) {
    match id {
        SYSCALL_READ => syscall_read(args[0], args[1], args[2]),
        SYSCALL_WRITE => syscall_write(args[0], args[1], args[2]),
        SYSCALL_EXIT => syscall_exit(args[0] as isize),
        SYSCALL_YIELD => syscall_yield(),
        _ => todo!(),
    }
}

pub fn syscall_read(fd: usize, buffer_ptr: usize, buffer_len: usize) {
    if fd != 0 {
        panic!("[syscall] Doesn't support file read.");
    }
    let task = fetch_curr_task();
    let task_guard = task.lock();
    let page_table = task_guard.user_mem().page_table();
    let mut buffer = translate_bytes(page_table, buffer_ptr as *mut u8, buffer_len);
    let stdin = Stdin;
    buffer.iter_mut().for_each(|b| **b = stdin.getchar() as u8);
}

pub fn syscall_write(fd: usize, buffer_ptr: usize, buffer_len: usize) {
    if fd != 1 {
        panic!("[syscall] Doesn't support file write.");
    }
    let task = fetch_curr_task();
    let task_guard = task.lock();
    let page_table = task_guard.user_mem().page_table();
    let buffer = translate_bytes(page_table, buffer_ptr as *const u8, buffer_len);
    let stdout = Stdout;
    buffer.iter().for_each(|&&mut b| stdout.putchar(b));
}

pub fn syscall_exit(exit_code: isize) {
    exit_and_yield(exit_code);
}

pub fn syscall_yield() {
    suspend_and_yield();
}
