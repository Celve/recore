#[macro_use]
pub mod stdout;

#[macro_use]
pub mod kout;

pub mod stdin;

use stdin::Stdin;
use stdout::Stdout;

use self::kout::Kout;

pub fn stdout() -> Stdout {
    Stdout
}

pub fn stdin() -> Stdin {
    Stdin
}

pub fn kout() -> Kout {
    Kout
}
