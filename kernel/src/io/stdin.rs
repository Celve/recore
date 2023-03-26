use super::uart::recv_from_uart;

pub struct Stdin;

impl Stdin {
    pub fn getchar(&self) -> u8 {
        let result = recv_from_uart();
        result
    }
}
