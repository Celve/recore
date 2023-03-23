#![no_main]
#![no_std]

use user::fork;

#[macro_use]
extern crate user;

#[no_mangle]
fn main() {
    if fork() == 0 {
        println!("This is parent!");
    } else {
        println!("This is children!");
    }
}
