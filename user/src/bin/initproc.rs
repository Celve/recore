#![no_main]
#![no_std]

#[macro_use]
extern crate user;

use user::{exec, fork, wait, yield_now};

#[no_mangle]
fn main() {
    if fork() == 0 {
        exec("shell\0");
    } else {
        let mut exit_code: i32 = 0;
        loop {
            let pid = wait(&mut exit_code);
            match pid {
                -1 => return,
                -2 => yield_now(),
                pid => println!(
                    "[initproc] Recycle child process {} with exit code {}.",
                    pid, exit_code
                ),
            }
        }
    }
}
