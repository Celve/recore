#![no_std]
#![no_main]

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

use core::fmt::Display;

use alloc::{string::String, vec::Vec};
use fosix::fs::OpenFlags;
use user::{chdir, console, exec, fork, mkdir, open, waitpid};

const BS: char = 8 as char;
const DL: char = 127 as char;

struct Path {
    inner: Vec<String>,
}

impl Path {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }

    fn push(&mut self, str: String) {
        self.inner.push(str);
    }

    fn pop(&mut self) {
        self.inner.pop();
    }

    fn last(&self) -> Option<&String> {
        self.inner.last()
    }

    fn len(&mut self) -> usize {
        self.inner.len()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut result = String::from("/");
        for s in &self.inner {
            result.push_str(s);
            result.push('/');
        }
        write!(f, "{}", result)
    }
}

fn getline() -> String {
    let mut c = console::stdin().getchar();
    let mut result = String::new();
    while c != '\n' && c != '\r' {
        if c == BS || c == DL {
            if !result.is_empty() {
                print!("{c}");
                result.pop();
            }
        } else {
            result.push(c);
            print!("{c}");
        }
        c = console::stdin().getchar();
    }
    result
}

#[no_mangle]
fn main() {
    let mut cwd: Path = Path::new();
    loop {
        print!("{} > ", cwd);
        let str = getline();
        if str.is_empty() {
            println!("");
            continue;
        }

        let args: Vec<&str> = str.split_whitespace().collect();
        let mut cargs: Vec<String> = args.iter().map(|s| String::from(*s)).collect();
        cargs.iter_mut().for_each(|s| s.push('\0'));
        let mut uargs: Vec<*const u8> = cargs.iter().map(|str| str.as_ptr()).collect();
        uargs.push(core::ptr::null());

        match args[0] {
            "cd" => {
                if args.len() == 1 {
                    println!("[user] cd: missing operand");
                } else {
                    let mut path = String::from(args[1]);
                    if path.ends_with("/") {
                        path.pop();
                    }
                    path.push('\0');
                    if chdir(path.as_str()) == -1 {
                        println!("[user] cd {}: No such file or directory", args[1]);
                    } else {
                        let splited: Vec<&str> = args[1].split('/').collect();
                        for s in splited {
                            if s == ".." {
                                cwd.pop();
                            } else if s == "." {
                                continue;
                            } else {
                                cwd.push(String::from(s));
                            }
                        }
                    }
                    println!("");
                }
            }

            "mkdir" => {
                if args.len() == 1 {
                    println!("[user] cd: missing operand");
                } else {
                    let mut path = String::from(args[1]);
                    if path.ends_with("/") {
                        path.pop();
                    }
                    path.push('\0');
                    let dfd = open(".\0", OpenFlags::DIR);
                    assert_ne!(dfd, -1);
                    if mkdir(dfd as usize, path.as_str()) == -1 {
                        println!("[user] mkdir {}: Fail to make such directory", args[1]);
                    } else {
                        println!("");
                    }
                }
            }

            _ => {
                let mut name = String::from(args[0]);
                name.push('\0');
                let pid = fork();
                if pid == 0 {
                    if exec(name.as_str(), &uargs) == -1 {
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
    }
}
