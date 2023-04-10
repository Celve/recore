use super::uart::recv_from_uart;

#[derive(Clone, Copy)]
pub struct Stdin;

impl Stdin {
    pub fn getchar(&self) -> u8 {
        let result = recv_from_uart();
        result
    }
}

impl Stdin {
    pub fn read(&self, buf: &mut [u8]) -> usize {
        buf.iter_mut().for_each(|b| *b = self.getchar());
        buf.len()
    }
}
