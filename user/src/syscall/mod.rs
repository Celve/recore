pub mod file;

use core::arch::asm;

use alloc::vec::Vec;
use fosix::syscall::*;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    // let ret: isize;
    // unsafe {
    //     asm!("ecall", inlateout("a0") args[0] => ret, );
    // }
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a7") id
        );
    }
    return ret;
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("[user] Return from syscall_exit()");
}

pub fn sys_yield() {
    syscall(SYSCALL_YIELD, [0; 3]);
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0; 3])
}

pub fn sys_exec(path: &str, args: &Vec<*const u8>) -> isize {
    syscall(
        SYSCALL_EXEC,
        [path.as_ptr() as usize, args.as_ptr() as usize, 0],
    )
}

pub fn sys_waitpid(pid: isize, exit_code: &mut i32) -> isize {
    syscall(
        SYSCALL_WAITPID,
        [pid as usize, exit_code as *mut i32 as usize, 0],
    )
}
