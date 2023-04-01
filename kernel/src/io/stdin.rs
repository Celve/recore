use crate::fs::{
    file::{File, Fileable},
    segment::Segment,
};

use super::uart::recv_from_uart;

pub struct Stdin;

impl Stdin {
    pub fn getchar(&self) -> u8 {
        let result = recv_from_uart();
        result
    }
}

impl Fileable for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        buf.iter_mut().for_each(|b| *b = self.getchar());
        buf.len()
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        unimplemented!()
    }

    fn seek(&mut self, offset: usize) {
        unimplemented!()
    }
}
