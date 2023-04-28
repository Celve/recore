#[macro_use]
pub mod stdout;

#[macro_use]
pub mod kout;

#[macro_use]
pub mod log;

pub mod stdin;

use kout::Kout;
use stdin::Stdin;
use stdout::Stdout;

use self::log::LogManager;

pub fn stdout() -> Stdout {
    Stdout
}

pub fn stdin() -> Stdin {
    Stdin
}

pub fn kout() -> Kout {
    Kout
}

pub fn log_manager() -> LogManager {
    LogManager::new()
}
