#![no_main]
#![no_std]

#[macro_use]
extern crate user;

#[no_mangle]
fn main() {
    let p: *mut u8 = 0 as *mut u8;
    unsafe {
        *p = 10;
    }
    println!("Hello, world!");
    loop {}
}
