#![no_std]
#![no_main]

#[macro_use]
extern crate user;
extern crate alloc;

use alloc::vec::Vec;
use user::{exit, mutex_create, mutex_lock, mutex_unlock, thread_create, waittid};

static mut mutex: usize = 0;
static mut A: usize = 0;
const PER_THREAD_DEFAULT: usize = 10000;
const THREAD_COUNT_DEFAULT: usize = 16;
static mut PER_THREAD: usize = 0;

unsafe fn critical_section(t: &mut usize) {
    mutex_lock(mutex);
    let a = &mut A as *mut usize;
    let cur = a.read_volatile();
    for _ in 0..500 {
        *t = (*t) * (*t) % 10007;
    }
    a.write_volatile(cur + 1);
    mutex_unlock(mutex);
}

unsafe fn f() -> ! {
    let mut t = 2usize;
    for _ in 0..PER_THREAD {
        critical_section(&mut t);
    }
    exit(t as i32)
}

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    // create mutex
    unsafe {
        mutex = mutex_create(false) as usize;
    }

    let mut thread_count = THREAD_COUNT_DEFAULT;
    let mut per_thread = PER_THREAD_DEFAULT;
    if argc >= 2 {
        thread_count = argv[1].parse().unwrap();
        if argc >= 3 {
            per_thread = argv[2].parse().unwrap();
        }
    }
    unsafe {
        PER_THREAD = per_thread;
    }
    let mut v = Vec::new();
    for _ in 0..thread_count {
        v.push(thread_create(f as usize, 0) as usize);
    }
    for tid in v.into_iter() {
        let exit_code: isize = 0;
        waittid(tid as isize, &exit_code as *const _ as usize);
    }
    println!(
        "suppose {}, find {}",
        unsafe { PER_THREAD } * thread_count,
        unsafe { A }
    );
    0
}
