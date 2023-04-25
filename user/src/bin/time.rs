#![no_std]
#![no_main]

use user::time;

#[macro_use]
extern crate user;

extern crate alloc;

#[no_mangle]
fn main() {
    println!("{}", time());
}
