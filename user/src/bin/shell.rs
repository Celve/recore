#![no_std]
#![no_main]

use user::{exec, fork, yield_now};

#[macro_use]
extern crate user;

#[no_mangle]
fn main() {
    loop {
        print!("> ");
        let str = user::console::stdin().getline();
        if fork() == 0 {
            if exec(str.as_str()) == -1 {
                println!("[user] Exec {} failed", str);
                return;
            }
        } else {
            yield_now();
        }
    }
}
