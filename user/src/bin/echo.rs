#![no_std]
#![no_main]

#[macro_use]
extern crate user;

extern crate alloc;

use alloc::string::String;
use user::console;

#[no_mangle]
fn main() {
    let mut c = console::Stdin.getchar();
    let mut str = String::new();
    while c != '\n' && c != '\r' {
        str.push(c);
        c = console::Stdin.getchar();
    }
    println!("{}", str);
}
