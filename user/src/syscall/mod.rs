pub mod file;
pub mod proc;

use core::arch::asm;

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
