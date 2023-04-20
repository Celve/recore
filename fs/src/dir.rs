use core::mem::size_of;

use alloc::{string::String, sync::Arc, vec::Vec};
use fosix::fs::{DirEntry, FileStat, OpenFlags};
use spin::{Spin, SpinGuard};

use crate::{disk::DiskManager, fuse::Fuse};

use super::{
    file::File,
    inode::{Inode, InodePtr, InodeType},
};

pub struct Dir<D: DiskManager> {
    inner: Arc<Spin<DirInner<D>>>,
}

pub struct DirInner<D: DiskManager> {
    myself: InodePtr<D>,
    fuse: Arc<Fuse<D>>,
}

impl<D: DiskManager> Clone for Dir<D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<D: DiskManager> Dir<D> {
    pub fn new(myself: InodePtr<D>, fuse: Arc<Fuse<D>>) -> Self {
        Self {
            inner: Arc::new(Spin::new(DirInner::new(myself, fuse))),
        }
    }

    pub fn lock(&self) -> SpinGuard<DirInner<D>> {
        self.inner.lock()
    }
}

impl<D: DiskManager> DirInner<D> {
    pub fn ls(&self) -> Vec<String> {
        let cache = self.fuse.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        let num_de = inode.size() / size_of::<DirEntry>();
        let mut names = Vec::new();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(
                de.as_bytes_mut(),
                i * size_of::<DirEntry>(),
                self.fuse.clone(),
            );
            names.push(String::from(de.name()));
        }
        names
    }

    pub fn cd(&self, name: &str) -> Option<Dir<D>> {
        let de = self.get_de(name)?;
        let inode_ptr = InodePtr::new(de.iid() as usize, self.fuse.clone());

        let blk = self.fuse.cache_manager().get(inode_ptr.bid());
        let blk_guard = blk.lock();
        let inode = &blk_guard.as_array::<Inode>()[inode_ptr.offset()];
        if inode.is_dir() {
            Some(Dir::new(inode_ptr, self.fuse.clone()))
        } else {
            None
        }
    }

    pub fn open(&self, name: &str, flags: OpenFlags) -> Option<File<D>> {
        let de = self.get_de(name);
        if let Some(de) = de {
            let inode_ptr = InodePtr::new(de.iid(), self.fuse.clone());

            let blk = self.fuse.cache_manager().get(inode_ptr.bid());
            let mut blk_guard = blk.lock();
            let inode = &mut blk_guard.as_array_mut::<Inode>()[inode_ptr.offset()];
            if inode.is_file() {
                if flags.contains(OpenFlags::TRUNC) {
                    inode.trunc(self.fuse.clone());
                }
                return Some(File::new(
                    inode_ptr,
                    self.myself.clone(),
                    flags.into(),
                    self.fuse.clone(),
                ));
            }
        } else if flags.contains(OpenFlags::CREATE) {
            self.touch(name).unwrap();
            let de = self.get_de(name).unwrap();
            return Some(File::new(
                InodePtr::new(de.iid(), self.fuse.clone()),
                self.myself.clone(),
                flags.into(),
                self.fuse.clone(),
            ));
        }
        None
    }

    pub fn mkdir(&self, name: &str) -> Result<(), ()> {
        self.create(name, InodeType::Directory)
    }

    pub fn touch(&self, name: &str) -> Result<(), ()> {
        self.create(name, InodeType::File)
    }

    pub fn to_dir_entries(&self) -> Vec<DirEntry> {
        let cache = self.fuse.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        let num_de = inode.size() / size_of::<DirEntry>();
        let mut des = Vec::new();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(
                de.as_bytes_mut(),
                i * size_of::<DirEntry>(),
                self.fuse.clone(),
            );
            des.push(de);
        }
        des
    }

    fn get_de(&self, name: &str) -> Option<DirEntry> {
        let cache = self.fuse.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        let num_de = inode.size() / size_of::<DirEntry>();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(
                de.as_bytes_mut(),
                i * size_of::<DirEntry>(),
                self.fuse.clone(),
            );
            if de.name() == name {
                return Some(de);
            }
        }
        None
    }

    fn create(&self, name: &str, ty: InodeType) -> Result<(), ()> {
        if self.ls().iter().any(|s| s == name) {
            Err(())
        } else {
            let iid = self.fuse.alloc_iid().unwrap();
            let inode_ptr = InodePtr::new(iid, self.fuse.clone());

            // modify inner inode
            {
                let cache = self.fuse.cache_manager().get(inode_ptr.bid());
                let mut cache_guard = cache.lock();
                let inode = &mut cache_guard.as_array_mut::<Inode>()[inode_ptr.offset()];
                *inode = match ty {
                    InodeType::File => Inode::empty_file(),
                    InodeType::Directory => {
                        Inode::empty_dir(iid, self.myself.iid(), self.fuse.clone())
                    }
                }
            }

            // modify outer inode
            {
                let cache = self.fuse.cache_manager().get(self.myself.bid());
                let mut cache_guard = cache.lock();
                let inode = &mut cache_guard.as_array_mut::<Inode>()[self.myself.offset()];
                inode.write_at_end(DirEntry::new(name, iid).as_bytes(), self.fuse.clone());
            }
            Ok(())
        }
    }
}

impl<D: DiskManager> DirInner<D> {
    pub fn new(myself: InodePtr<D>, fuse: Arc<Fuse<D>>) -> Self {
        Self { myself, fuse }
    }

    pub fn stat(&self) -> FileStat {
        let cache = self.fuse.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        FileStat::new(inode.size())
    }
}
