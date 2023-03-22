#[macro_use]
pub mod stdout;

pub mod uart;

use stdout::Stdout;

pub fn stdout() -> Stdout {
    Stdout
}

// TODO: construct stdin() function
