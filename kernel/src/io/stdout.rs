use super::uart::send_to_uart;
use core::fmt::{Arguments, Write};

pub struct Stdout;

impl Stdout {
    pub fn putchar(&self, c: char) {
        send_to_uart(c as u8);
    }

    pub fn print(&mut self, args: Arguments) {
        self.write_fmt(args).unwrap();
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().for_each(|c| self.putchar(c));
        Ok(())
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
