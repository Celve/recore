use crate::drivers::uart::UARTK;

use core::fmt::{Arguments, Write};

#[derive(Clone, Copy)]
pub struct Kout;

impl Kout {
    pub fn putchar(&self, c: u8) {
        UARTK.send(c);
    }

    pub fn print(&mut self, args: Arguments) {
        self.write_fmt(args).unwrap();
    }
}

impl Write for Kout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().for_each(|c| self.putchar(c as u8));
        Ok(())
    }
}

impl Kout {
    pub fn write(&mut self, buf: &[u8]) -> usize {
        buf.iter().for_each(|b| self.putchar(*b));
        buf.len()
    }
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::kout().print(format_args!($fmt $($t)*))
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::kout().print(format_args!(concat!($fmt, "\n") $($t)*))
    };
}
