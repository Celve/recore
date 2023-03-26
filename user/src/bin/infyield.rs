#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::yield_now;

#[no_mangle]
fn main() {
    loop {
        yield_now();
    }
}
