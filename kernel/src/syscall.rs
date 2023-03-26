use crate::{
    io::{stdin::Stdin, stdout::Stdout},
    task::{
        exit_and_yield, loader::get_app_data, manager::MANAGER, processor::fetch_curr_task,
        suspend_and_yield, task::TaskStatus,
    },
};

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_READ => syscall_read(args[0], args[1], args[2]),
        SYSCALL_WRITE => syscall_write(args[0], args[1], args[2]),
        SYSCALL_EXIT => syscall_exit(args[0] as isize),
        SYSCALL_YIELD => syscall_yield(),
        SYSCALL_FORK => syscall_fork(),
        SYSCALL_EXEC => syscall_exec(args[0]),
        SYSCALL_WAITPID => syscall_waitpid(args[0] as isize, args[1]),
        _ => todo!(),
    }
}

pub fn syscall_read(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
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

pub fn syscall_write(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
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

pub fn syscall_exit(exit_code: isize) -> isize {
    exit_and_yield(exit_code);
    0
}

pub fn syscall_yield() -> isize {
    suspend_and_yield();
    0
}

pub fn syscall_fork() -> isize {
    let task = fetch_curr_task().fork();
    let pid = task.lock().pid();
    *task.lock().trap_ctx_mut().a0_mut() = 0;
    MANAGER.lock().push(task);
    println!("[kernel] Fork a new process with pid {}.", pid);
    pid as isize
}

pub fn syscall_exec(path: usize) -> isize {
    let s = fetch_curr_task()
        .lock()
        .user_mem()
        .page_table()
        .translate_str(path.into());
    if let Some(elf_data) = get_app_data(s.as_str()) {
        println!("[kernel] Exec a new program.");
        fetch_curr_task().exec(elf_data);
        0
    } else {
        println!("[kernel] Fail to exec {}.", s);
        -1
    }
}

pub fn syscall_waitpid(pid: isize, exit_code_ptr: usize) -> isize {
    let task = fetch_curr_task();
    let mut task_guard = task.lock();

    // find satisfied children
    let result = task_guard.children().iter().position(|task| {
        let task = task.lock();
        (pid == -1 || pid as usize == task.pid()) && *task.task_status() == TaskStatus::Zombie
    });

    return if let Some(pos) = result {
        let removed_task = task_guard.children_mut().remove(pos);
        *task_guard
            .user_mem()
            .page_table()
            .translate_any::<isize>(exit_code_ptr.into()) = removed_task.lock().exit_code();
        let pid = removed_task.lock().pid() as isize;
        pid
    } else if task_guard
        .children()
        .iter()
        .any(|task| pid == -1 || task.lock().pid() == pid as usize)
    {
        -2
    } else {
        -1
    };
}
