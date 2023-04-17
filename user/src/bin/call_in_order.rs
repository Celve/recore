#![no_std]
#![no_main]

use alloc::vec::Vec;
use user::*;

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

const NUM_THREADS: usize = 5;

static mut MUTEX: usize = 0;
static mut CONDVAR: usize = 0;
static mut CURRENT: usize = 0;

fn task(id: usize) {
    mutex_lock(unsafe { MUTEX });
    loop {
        condvar_wait(unsafe { CONDVAR }, unsafe { MUTEX });
        if unsafe { CURRENT } == id {
            break;
        }
    }
    println!("This is thread {}", id);
    unsafe {
        CURRENT = 0;
    }
    exit(0);
}

#[no_mangle]
fn main() {
    unsafe {
        MUTEX = mutex_create(true) as usize;
        CONDVAR = condvar_create() as usize;
        CURRENT = 0;
    }

    let mut tasks = Vec::new();
    for i in 1..=NUM_THREADS {
        tasks.push(thread_create(task as usize, i) as usize);
    }

    for i in (1..=NUM_THREADS).rev() {
        unsafe {
            CURRENT = i;
        }
        mutex_lock(unsafe { MUTEX });
        while unsafe { CURRENT } != 0 {
            mutex_unlock(unsafe { MUTEX });
            condvar_notify_all(unsafe { CONDVAR });
            yield_now();
        }
        mutex_unlock(unsafe { MUTEX });
    }

    for task in tasks {
        let exit_code: usize = 0;
        waittid(task as isize, &exit_code as *const _ as usize);
    }
}
