#![no_main]
#![no_std]

use user::sleep;

#[macro_use]
extern crate user;

#[no_mangle]
fn main() {
    println!("Begin to sleep!");
    sleep(3000);
    println!("I now finish sleep!");
}
