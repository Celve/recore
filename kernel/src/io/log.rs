use core::fmt::{Arguments, Write};

use lazy_static::lazy_static;
use spin::Spin;

use crate::{config::VIRT_UART, drivers::uart::UartRaw};

const FATAL: &str = "\x1b[31m";
const WARN: &str = "\x1b[93m";
const INFO: &str = "\x1b[34m";
const DEBUG: &str = "\x1b[32m";
const TRACE: &str = "\x1b[90m";
const END: &str = "\x1b[0m";
const NEWLINE: &str = "\n";

/// A manager that provides interface for the log system.
///
/// It's promised that the lines of different log would not interleave due to the inner lock.
///
/// The spin lock is used because the mcs is depending on the cache manager. However, the demand for log is always high.
pub struct LogManager {
    uart: Spin<UartRaw>,
}

lazy_static! {
    pub static ref LOG_MANAGER: LogManager = LogManager::new();
}

impl LogManager {
    pub fn new() -> Self {
        Self {
            uart: Spin::new(UartRaw::new(VIRT_UART)),
        }
    }

    pub fn print(&mut self, args: Arguments) {
        self.write_fmt(args).unwrap();
    }
}

impl Write for LogManager {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let uart = self.uart.lock();
        s.as_bytes().iter().for_each(|c| uart.send(*c));
        Ok(())
    }
}

#[macro_export]
macro_rules! fatal {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(FATAL, $fmt, END)))
    };
}

#[macro_export]
macro_rules! fatalln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(FATAL, $fmt, END, NEWLINE) $($t)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(WARN, $fmt, END)))
    };
}

#[macro_export]
macro_rules! warnln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(WARN, $fmt, END, NEWLINE) $($t)*))
    };
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(INFO, $fmt, END)))
    };
}

#[macro_export]
macro_rules! infoln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(INFO, $fmt, END, NEWLINE) $($t)*))
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(DEBUG, $fmt, END)))
    };
}

#[macro_export]
macro_rules! debugln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(DEBUG, $fmt, END, NEWLINE) $($t)*))
    };
}

#[macro_export]
macro_rules! trace {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(TRACE, $fmt, END)))
    };
}

#[macro_export]
macro_rules! traceln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log::LOG_MANAGER.print(format_args!(concat!(TRACE, $fmt, END, NEWLINE) $($t)*))
    };
}
