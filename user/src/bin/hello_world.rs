#![no_std]
#![no_main]

use user::yield_now;

#[macro_use]
extern crate user;

#[no_mangle]
fn main() {
    println!("Hello, world!");
}
