use super::{stdout::Stdout, uart::receive_from_uart};

pub struct Stdin;

impl Stdin {
    pub fn getchar(&self) -> u8 {
        let result = receive_from_uart();
        Stdout.putchar(result);
        result
    }
}
