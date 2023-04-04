use bitflags::bitflags;
use fosix::fs::{FilePerm, FileStat, OpenFlags};

use super::{
    cache::CACHE_MANAGER,
    dir::Dir,
    inode::{Inode, InodePtr},
    segment::Segment,
};

pub struct File {
    myself: InodePtr,
    parent: InodePtr,
    offset: usize,
    perm: FilePerm,
}

impl File {
    pub fn read_at(&self, buf: &mut [u8], offset: usize) -> usize {
        if !self.perm.contains(FilePerm::READABLE) {
            return 0;
        }

        let cache = CACHE_MANAGER.lock().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        inode.read_at(buf, offset)
    }

    pub fn write_at(&self, buf: &[u8], offset: usize) -> usize {
        if !self.perm.contains(FilePerm::WRITEABLE) {
            return 0;
        }

        let cache = CACHE_MANAGER.lock().get(self.myself.bid());
        let mut cache_guard = cache.lock();
        let inode = &mut cache_guard.as_array_mut::<Inode>()[self.myself.offset()];

        inode.write_at(buf, offset)
    }

    pub fn size(&self) -> usize {
        let cache = CACHE_MANAGER.lock().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        inode.size()
    }

    pub fn parent(&self) -> Dir {
        Dir::new(self.parent)
    }
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

    pub fn stat(&self) -> FileStat {
        FileStat::new(self.size())
    }
}

impl File {
    pub fn new(myself: InodePtr, parent: InodePtr, perm: FilePerm) -> Self {
        Self {
            myself,
            parent,
            offset: 0,
            perm,
        }
    }
}
