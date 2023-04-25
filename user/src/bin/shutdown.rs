#![no_std]
#![no_main]

use user::shutdown;

#[macro_use]
extern crate user;

extern crate alloc;

#[no_mangle]
fn main(argc: usize, argv: &[&str]) {
    if argc != 2 {
        println!("Usage: shutdown <exit_code>");
        return;
    }
    let exit_code = if let Ok(parsed) = argv[1].parse::<usize>() {
        parsed
    } else {
        println!("Usage: shutdown <exit_code>");
        return;
    };
    shutdown(exit_code);
}
