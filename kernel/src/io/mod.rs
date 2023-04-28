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

use self::log::{LogManager, LOG_MANAGER};

pub fn stdout() -> Stdout {
    Stdout
}

pub fn stdin() -> Stdin {
    Stdin
}

pub fn kout() -> Kout {
    Kout
}
