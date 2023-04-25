#![no_std]
#![no_main]

use user::procdump;

#[macro_use]
extern crate user;

extern crate alloc;

#[no_mangle]
fn main() {
    procdump();
}
