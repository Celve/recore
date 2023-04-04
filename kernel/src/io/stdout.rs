use crate::fs::{file::File, segment::Segment};

use super::uart::send_to_uart;
use core::fmt::{Arguments, Write};

#[derive(Clone, Copy)]
pub struct Stdout;

impl Stdout {
    pub fn putchar(&self, c: u8) {
        send_to_uart(c);
    }

    pub fn print(&mut self, args: Arguments) {
        self.write_fmt(args).unwrap();
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().for_each(|c| self.putchar(c as u8));
        Ok(())
    }
}

impl Stdout {
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        unimplemented!()
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        buf.iter().for_each(|b| self.putchar(*b));
        buf.len()
    }
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::stdout().print(format_args!($fmt $($t)*));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::stdout().print(format_args!(concat!($fmt, "\n") $($t)*));
    };
}
