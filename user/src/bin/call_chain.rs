#![no_std]
#![no_main]

use user::{exit, semaphore_create, semaphore_down, semaphore_up, thread_create, waittid};

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

static mut SEMAPHORE_AB: usize = 0;
static mut SEMAPHORE_BC: usize = 0;

fn threada() {
    println!("Now, it's thread a.");
    semaphore_up(unsafe { SEMAPHORE_AB });
    exit(0);
}

fn threadb() {
    semaphore_down(unsafe { SEMAPHORE_AB });
    println!("Now, it's thread b.");
    semaphore_up(unsafe { SEMAPHORE_BC });
    exit(0);
}

fn threadc() {
    semaphore_down(unsafe { SEMAPHORE_BC });
    println!("Now, it's thread c.");
    exit(0);
}

#[no_mangle]
fn main() {
    unsafe {
        SEMAPHORE_AB = semaphore_create(0) as usize;
        SEMAPHORE_BC = semaphore_create(0) as usize;
    }
    let threads = vec![
        thread_create(threadc as usize, 0),
        thread_create(threadb as usize, 0),
        thread_create(threada as usize, 0),
    ];
    for tid in threads {
        let exit_code: isize = 0;
        waittid(tid as isize, &exit_code as *const _ as usize);
    }
}
