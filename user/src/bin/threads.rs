#![no_std]
#![no_main]

use user::{exit, thread_create, waittid, yield_now};

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

pub fn thread_a() -> ! {
    for i in 0..1000 {
        print!("a");
        if i == 500 {
            yield_now();
        }
    }
    exit(1)
}

pub fn thread_b() -> ! {
    for i in 0..1000 {
        print!("b");
        if i == 500 {
            yield_now();
        }
    }
    exit(2)
}

pub fn thread_c() -> ! {
    for i in 0..1000 {
        print!("c");
        if i == 500 {
            yield_now();
        }
    }
    exit(3)
}

#[no_mangle]
pub fn main() -> i32 {
    let v = vec![
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];
    for tid in v.iter() {
        let exit_code = 0;
        waittid(*tid, &exit_code as *const i32 as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    println!("main thread exited.");
    0
}
