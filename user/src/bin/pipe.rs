#![no_main]
#![no_std]

use user::{close, fork, pipe, read, write};

#[macro_use]
extern crate user;

#[no_mangle]
fn main() {
    let mut fds = [0usize; 2];
    pipe(&mut fds);

    if fork() == 0 {
        close(fds[0]);
        write(fds[1], b"Hello, world!\n");
    } else {
        close(fds[1]);
        let mut bytes = 0;
        while bytes < 14 {
            let mut buf = [0u8; 1];
            bytes += read(fds[0], &mut buf);
            print!("{}", buf[0] as char);
        }
    }
}
