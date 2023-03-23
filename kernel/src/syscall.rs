use alloc::sync::Arc;
use spin::mutex::Mutex;

use crate::{
    io::{stdin::Stdin, stdout::Stdout},
    mm::page_table::translate_bytes,
    task::{exit_and_yield, manager::MANAGER, processor::fetch_curr_task, suspend_and_yield},
};

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;

pub fn syscall(id: usize, args: [usize; 3]) {
    match id {
        SYSCALL_READ => syscall_read(args[0], args[1], args[2]),
        SYSCALL_WRITE => syscall_write(args[0], args[1], args[2]),
        SYSCALL_EXIT => syscall_exit(args[0] as isize),
        SYSCALL_YIELD => syscall_yield(),
        SYSCALL_FORK => syscall_fork(),
        SYSCALL_EXEC => syscall_exec(args[0]),
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
    let buffer = translate_bytes(
        fetch_curr_task().lock().user_mem().page_table(),
        buffer_ptr as *const u8,
        buffer_len,
    );
    let stdout = Stdout;
    buffer.iter().for_each(|&&mut b| stdout.putchar(b));
}

pub fn syscall_exit(exit_code: isize) {
    exit_and_yield(exit_code);
}

pub fn syscall_yield() {
    suspend_and_yield();
}

pub fn syscall_fork() {
    let task = fetch_curr_task();
    let new_task = Arc::new(Mutex::new(task.lock().clone()));

    // do the copy with a0 modified
    {
        let new_task_guard = new_task.lock();
        let new_task_pid = new_task_guard.pid();
        new_task_guard.trap_ctx_mut().saved_regs[10] = new_task_pid;
    }

    // modify the original task's a0
    task.lock().trap_ctx_mut().saved_regs[10] = 0;

    // make new process the original's children
    task.lock().children_mut().push(new_task.clone());
    *new_task.lock().parent_mut() = Some(Arc::downgrade(&task));

    MANAGER.lock().push(new_task);
}

pub fn syscall_exec(id: usize) {
    fetch_curr_task().lock().exec(id);
}
