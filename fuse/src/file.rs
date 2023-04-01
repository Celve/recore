use spin::mutex::Mutex;
use std::sync::Arc;

use super::inode::Inode;

pub struct File {
    inode: Arc<Mutex<Inode>>,
    offset: usize,
}

impl File {
    pub fn seek(&mut self, new_offset: usize) {
        self.offset = new_offset;
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let bytes = self.read_at(buf, self.offset);
        self.offset += bytes;
        bytes
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        let bytes = self.write_at(buf, self.offset);
        self.offset += bytes;
        bytes
    }

    pub fn read_at(&self, buf: &mut [u8], offset: usize) -> usize {
        self.inode.lock().read_at(buf, offset)
    }

    pub fn write_at(&self, buf: &[u8], offset: usize) -> usize {
        self.inode.lock().write_at(buf, offset)
    }
}

impl File {
    pub fn new(inode: Arc<Mutex<Inode>>) -> Self {
        Self { inode, offset: 0 }
    }
}
