use crate::{
    io::{stdin::Stdin, stdout::Stdout},
    task::processor::fetch_curr_task,
};

pub fn sys_read(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    if fd != 0 {
        panic!("[syscall] Doesn't support file read.");
    }
    let mut buffer = {
        let task = fetch_curr_task();
        let task_guard = task.lock();
        let page_table = task_guard.user_mem().page_table();
        page_table.translate_bytes(buffer_ptr.into(), buffer_len)
    };
    let stdin = Stdin;
    buffer.iter_mut().for_each(|b| **b = stdin.getchar() as u8);
    buffer_len as isize
}

pub fn sys_write(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    if fd != 1 {
        panic!("[syscall] Doesn't support file write.");
    }
    let buffer = fetch_curr_task()
        .lock()
        .user_mem()
        .page_table()
        .translate_bytes(buffer_ptr.into(), buffer_len);
    let stdout = Stdout;
    buffer.iter().for_each(|&&mut b| stdout.putchar(b));
    buffer_len as isize
}
