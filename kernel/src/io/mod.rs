#[macro_use]
pub mod stdout;
pub mod stdin;

use stdin::Stdin;
use stdout::Stdout;

pub fn stdout() -> Stdout {
    Stdout
}

pub fn stdin() -> Stdin {
    Stdin
}
