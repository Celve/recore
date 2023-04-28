use core::fmt::{Arguments, Write};

use lazy_static::lazy_static;
use spin::Spin;

use crate::{
    config::{LOG_LEVEL, VIRT_UART},
    drivers::uart::UartRaw,
};

/// A manager that provides interface for the log system.
///
/// It's promised that the lines of different log would not interleave due to the inner lock.
///
/// The spin lock is used because the mcs is depending on the cache manager. However, the demand for log is always high.
pub struct LogManager {
    uart: UartRaw,
    level: LogLevel,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Fatal,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogManager {
    pub fn new() -> Self {
        Self {
            uart: UartRaw::new(VIRT_UART),
            level: LOG_LEVEL,
        }
    }

    pub fn print(&mut self, level: LogLevel, args: Arguments) {
        if level <= self.level {
            self.write_fmt(args).unwrap();
        }
    }
}
impl Write for LogManager {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|c| self.uart.send(*c));
        Ok(())
    }
}

#[macro_export]
macro_rules! fatal {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().fatal(
            $crate::io::log::LogLevel::Fatal,
            format_args!(concat!("\x1b[31m", "[FATAL] ", $fmt, "\x1b[0m")),
        )
    };
}

#[macro_export]
macro_rules! fatalln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print($crate::io::log::LogLevel::Fatal, format_args!(concat!("\x1b[31m", "[FATAL] ", $fmt, "\x1b[0m", "\n") $($t)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print(
            $crate::io::log::LogLevel::Warn,
            format_args!(concat!("\x1b[93m", "[WARN] ", $fmt, "\x1b[0m")),
        )
    };
}

#[macro_export]
macro_rules! warnln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print($crate::io::log::LogLevel::Warn, format_args!(concat!("\x1b[93m", "[WARN] ", $fmt, "\x1b[0m", "\n") $($t)*))
    };
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print(
            $crate::io::log::LogLevel::Info,
            format_args!(concat!("\x1b[34m", "[INFO] ", $fmt, "\x1b[0m")),
        )
    };
}

#[macro_export]
macro_rules! infoln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print($crate::io::log::LogLevel::Info, format_args!(concat!("\x1b[34m", "[INFO] ", $fmt, "\x1b[0m", "\n") $($t)*))
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print(
            $crate::io::log::LogLevel::Debug,
            format_args!(concat!("\x1b[32m", "[DEBUG] ", $fmt, "\x1b[0m")),
        )
    };
}

#[macro_export]
macro_rules! debugln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print($crate::io::log::LogLevel::Debug, format_args!(concat!("\x1b[32m", "[DEBUG] ", $fmt, "\x1b[0m", "\n") $($t)*))
    };
}

#[macro_export]
macro_rules! trace {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print(
            $crate::io::log::LogLevel::trace,
            format_args!(concat!("\x1b[90m", "[TRACE] ", $fmt, "\x1b[0m")),
        )
    };
}

#[macro_export]
macro_rules! traceln {
    ($fmt: literal $($t: tt)*) => {
        $crate::io::log_manager().print($crate::io::log::LogLevel::trace, format_args!(concat!("\x1b[90m", "[TRACE] ", $fmt, "\x1b[0m", "\n") $($t)*))
    };
}
