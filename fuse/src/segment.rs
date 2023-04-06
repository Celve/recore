use std::vec::Vec;

pub struct Segment {
    inner: Vec<&'static mut [u8]>,
}

impl Segment {
    pub fn new(inner: Vec<&'static mut [u8]>) -> Self {
        Self { inner }
    }

    pub fn iter(&self) -> core::slice::Iter<&'static mut [u8]> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<&'static mut [u8]> {
        self.inner.iter_mut()
    }
}
