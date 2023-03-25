#![no_std]
#![no_main]

use user::{exec, fork, waitpid, yield_now};

#[macro_use]
extern crate user;

#[no_mangle]
fn main() {
    loop {
        print!("> ");
        let str = user::console::stdin().getline();
        let pid = fork();
        if pid == 0 {
            if exec(str.as_str()) == -1 {
                println!("[user] Exec {} failed", str);
                return;
            }
        } else {
            let mut exit_code: i32 = 0;
            waitpid(pid, &mut exit_code);
            println!("[user] Process {} exit with code {}", pid, exit_code);
        }
    }
}
