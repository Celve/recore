use bitflags::bitflags;
use fosix::fs::{FilePerm, FileStat, OpenFlags};

use super::{
    cache::CACHE_MANAGER,
    dir::Dir,
    inode::{Inode, InodePtr},
    segment::Segment,
};

pub trait Fileable: Send + Sync {
    fn seek(&mut self, new_offset: usize);

    fn read(&mut self, buf: &mut [u8]) -> usize;

    fn write(&mut self, buf: &[u8]) -> usize;

    fn stat(&self) -> FileStat;

    fn read_seg(&mut self, seg: &mut Segment) -> usize {
        let mut bytes = 0;
        for buf in seg.iter_mut() {
            bytes += self.read(buf);
        }
        bytes
    }

    fn write_seg(&mut self, seg: &Segment) -> usize {
        let mut bytes = 0;
        for buf in seg.iter() {
            bytes += self.write(buf);
        }
        bytes
    }
}

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

impl Fileable for File {
    fn seek(&mut self, new_offset: usize) {
        self.offset = new_offset;
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        let bytes = self.read_at(buf, self.offset);
        self.offset += bytes;
        bytes
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        let bytes = self.write_at(buf, self.offset);
        self.offset += bytes;
        bytes
    }

    fn stat(&self) -> FileStat {
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
