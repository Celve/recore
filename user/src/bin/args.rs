#![no_std]
#![no_main]

#[macro_use]
extern crate user;

extern crate alloc;

use alloc::string::String;
use user::console;

#[no_mangle]
fn main(argc: usize, argv: &[&str]) {
    println!("argc: {}", argc);
    for i in 0..argc {
        println!("argv[{}]: {}", i, argv[i]);
    }
}
