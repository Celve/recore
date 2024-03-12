use spin::SpinGuard;

use alloc::sync::Arc;
use fosix::fs::{FilePerm, FileStat, SeekFlag};
use spin::Spin;

use crate::{disk::DiskManager, fs::FileSys};

use super::{
    dir::Dir,
    inode::{Inode, InodePtr},
};

pub struct File<D: DiskManager> {
    inner: Arc<Spin<FileInner<D>>>,
}

pub struct FileInner<D: DiskManager> {
    myself: InodePtr<D>,
    parent: InodePtr<D>,
    offset: usize,
    perm: FilePerm,
    fs: Arc<FileSys<D>>,
}

impl<D: DiskManager> Clone for File<D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<D: DiskManager> File<D> {
    pub fn new(
        myself: InodePtr<D>,
        parent: InodePtr<D>,
        perm: FilePerm,
        fs: Arc<FileSys<D>>,
    ) -> Self {
        Self {
            inner: Arc::new(Spin::new(FileInner::new(myself, parent, perm, fs))),
        }
    }

    pub fn lock(&self) -> SpinGuard<FileInner<D>> {
        self.inner.lock()
    }
}

impl<D: DiskManager> FileInner<D> {
    pub fn read_at(&self, buf: &mut [u8], offset: usize) -> usize {
        if !self.perm.contains(FilePerm::READABLE) {
            return 0;
        }

        let cache = self.fs.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = unsafe { &cache_guard.as_array::<Inode>()[self.myself.offset()] };

        inode.read_at(buf, offset, self.fs.clone())
    }

    pub fn write_at(&self, buf: &[u8], offset: usize) -> usize {
        if !self.perm.contains(FilePerm::WRITEABLE) {
            return 0;
        }

        let cache = self.fs.cache_manager().get(self.myself.bid());
        let mut cache_guard = cache.lock();
        let inode = &mut cache_guard.as_array_mut::<Inode>()[self.myself.offset()];

        inode.write_at(buf, offset, self.fs.clone())
    }

    pub fn trunc(&mut self) -> usize {
        if !self.perm.contains(FilePerm::WRITEABLE) {
            return 0;
        }

        let cache = self.fs.cache_manager().get(self.myself.bid());
        let mut cache_guard = cache.lock();
        let inode = &mut cache_guard.as_array_mut::<Inode>()[self.myself.offset()];

        self.offset = 0;
        inode.trunc(self.fs.clone())
    }

    pub fn size(&self) -> usize {
        let cache = self.fs.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = unsafe { &cache_guard.as_array::<Inode>()[self.myself.offset()] };

        inode.size()
    }

    pub fn parent(&self) -> Dir<D> {
        Dir::new(self.parent.clone(), self.fs.clone())
    }
}

impl<D: DiskManager> FileInner<D> {
    pub fn seek(&mut self, new_offset: usize, flag: SeekFlag) {
        match flag {
            SeekFlag::SET => self.offset = new_offset,
            SeekFlag::CUR => self.offset += new_offset,
            SeekFlag::END => self.offset = self.size() + new_offset,
            _ => {}
        }
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

impl<D: DiskManager> FileInner<D> {
    pub fn new(
        myself: InodePtr<D>,
        parent: InodePtr<D>,
        perm: FilePerm,
        fs: Arc<FileSys<D>>,
    ) -> Self {
        Self {
            myself,
            parent,
            offset: 0,
            perm,
            fs,
        }
    }
}
