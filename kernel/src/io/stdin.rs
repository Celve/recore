use crate::drivers::uart::UART;

#[derive(Clone, Copy)]
pub struct Stdin;

impl Stdin {
    pub fn getchar(&self) -> u8 {
        UART.read()
    }
}

impl Stdin {
    pub fn read(&self, buf: &mut [u8]) -> usize {
        buf.iter_mut().for_each(|b| *b = self.getchar());
        buf.len()
    }
}
