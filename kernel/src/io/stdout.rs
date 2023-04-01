use crate::fs::{
    file::{File, Fileable},
    segment::Segment,
};

use super::uart::send_to_uart;
use core::fmt::{Arguments, Write};

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

impl Fileable for Stdout {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        unimplemented!()
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        buf.iter().for_each(|b| self.putchar(*b));
        buf.len()
    }

    fn seek(&mut self, offset: usize) {
        unimplemented!()
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
