use core::fmt::Write;

use crate::syscall::syscall_write;

const STDIN: usize = 0;
const STDOUT: usize = 1;

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        syscall_write(STDOUT, s.as_bytes());
        Ok(())
    }
}

impl Stdout {
    pub fn print(&mut self, args: core::fmt::Arguments) {
        self.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $($t: tt)*) => {
        Stdout.print(format_args!($fmt $($t)*));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $($t: tt)*) => {
        $crate::console::Stdout.print(format_args!(concat!($fmt, "\n") $($t)*));
    };
}
