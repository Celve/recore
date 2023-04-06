#![no_main]
#![no_std]

use alloc::vec::Vec;
use user::{exec, fork};

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

#[no_mangle]
fn main() {
    if fork() == 0 {
        println!("This is parent!");
    } else {
        exec("./hello_world\0", &vec![0 as *const u8]);
    }
}
