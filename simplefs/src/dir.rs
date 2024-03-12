use core::mem::size_of;

use alloc::{string::String, sync::Arc, vec::Vec};
use fosix::fs::{DirEntry, FileStat, OpenFlags, DIR_ENTRY_NAME_LEN};
use spin::{Spin, SpinGuard};

use crate::{disk::DiskManager, fs::FileSys};

use super::{
    file::File,
    inode::{Inode, InodePtr, InodeType},
};

pub struct Dir<D: DiskManager> {
    inner: Arc<Spin<DirInner<D>>>,
}

pub struct DirInner<D: DiskManager> {
    myself: InodePtr<D>,
    fs: Arc<FileSys<D>>,
}

impl<D: DiskManager> Clone for Dir<D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<D: DiskManager> Dir<D> {
    pub fn new(myself: InodePtr<D>, fs: Arc<FileSys<D>>) -> Self {
        Self {
            inner: Arc::new(Spin::new(DirInner::new(myself, fs))),
        }
    }

    pub fn lock(&self) -> SpinGuard<DirInner<D>> {
        self.inner.lock()
    }
}

impl<D: DiskManager> DirInner<D> {
    pub fn ls(&self) -> Vec<String> {
        let cache = self.fs.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = unsafe { &cache_guard.as_array::<Inode>()[self.myself.offset()] };

        let num_de = inode.size() / size_of::<DirEntry>();
        let mut names = Vec::new();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(
                de.as_bytes_mut(),
                i * size_of::<DirEntry>(),
                self.fs.clone(),
            );
            names.push(String::from(de.name()));
        }
        names
    }

    pub fn cd(&self, name: &str) -> Option<Dir<D>> {
        let de = self.get_de(name)?;
        let inode_ptr = InodePtr::new(de.iid() as usize, self.fs.clone());

        let blk = self.fs.cache_manager().get(inode_ptr.bid());
        let blk_guard = blk.lock();
        let inode = unsafe { &blk_guard.as_array::<Inode>()[inode_ptr.offset()] };
        if inode.is_dir() {
            Some(Dir::new(inode_ptr, self.fs.clone()))
        } else {
            None
        }
    }

    pub fn open(&self, name: &str, flags: OpenFlags) -> Option<File<D>> {
        let de = self.get_de(name);
        if let Some(de) = de {
            let inode_ptr = InodePtr::new(de.iid(), self.fs.clone());

            let blk = self.fs.cache_manager().get(inode_ptr.bid());
            let mut blk_guard = blk.lock();
            let inode = &mut blk_guard.as_array_mut::<Inode>()[inode_ptr.offset()];
            if inode.is_file() {
                if flags.contains(OpenFlags::TRUNC) {
                    inode.trunc(self.fs.clone());
                }
                return Some(File::new(
                    inode_ptr,
                    self.myself.clone(),
                    flags.into(),
                    self.fs.clone(),
                ));
            }
        } else if flags.contains(OpenFlags::CREATE) {
            self.touch(name).ok()?;
            let de = self.get_de(name).unwrap();
            return Some(File::new(
                InodePtr::new(de.iid(), self.fs.clone()),
                self.myself.clone(),
                flags.into(),
                self.fs.clone(),
            ));
        }
        None
    }

    fn is_valid_name(name: &str) -> bool {
        name != "." && name != ".." && !name.contains('/') && name.len() <= DIR_ENTRY_NAME_LEN
    }

    pub fn mkdir(&self, name: &str) -> Result<(), ()> {
        self.create(name, InodeType::Directory)
    }

    pub fn touch(&self, name: &str) -> Result<(), ()> {
        self.create(name, InodeType::File)
    }

    pub fn to_dir_entries(&self) -> Vec<DirEntry> {
        let cache = self.fs.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = unsafe { &cache_guard.as_array::<Inode>()[self.myself.offset()] };

        let num_de = inode.size() / size_of::<DirEntry>();
        let mut des = Vec::new();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(
                de.as_bytes_mut(),
                i * size_of::<DirEntry>(),
                self.fs.clone(),
            );
            des.push(de);
        }
        des
    }

    fn get_de(&self, name: &str) -> Option<DirEntry> {
        let cache = self.fs.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = unsafe { &cache_guard.as_array::<Inode>()[self.myself.offset()] };

        let num_de = inode.size() / size_of::<DirEntry>();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(
                de.as_bytes_mut(),
                i * size_of::<DirEntry>(),
                self.fs.clone(),
            );
            if de.name() == name {
                return Some(de);
            }
        }
        None
    }

    fn create(&self, name: &str, ty: InodeType) -> Result<(), ()> {
        if !Self::is_valid_name(name) || self.ls().iter().any(|s| s == name) {
            Err(())
        } else {
            let iid = self.fs.alloc_iid().unwrap();
            let inode_ptr = InodePtr::new(iid, self.fs.clone());

            // modify inner inode
            {
                let cache = self.fs.cache_manager().get(inode_ptr.bid());
                let mut cache_guard = cache.lock();
                let inode = &mut cache_guard.as_array_mut::<Inode>()[inode_ptr.offset()];
                *inode = match ty {
                    InodeType::File => Inode::empty_file(),
                    InodeType::Directory => {
                        Inode::empty_dir(iid, self.myself.iid(), self.fs.clone())
                    }
                }
            }

            // modify outer inode
            {
                let cache = self.fs.cache_manager().get(self.myself.bid());
                let mut cache_guard = cache.lock();
                let inode = &mut cache_guard.as_array_mut::<Inode>()[self.myself.offset()];
                inode.write_at_end(DirEntry::new(name, iid).as_bytes(), self.fs.clone());
            }
            Ok(())
        }
    }
}

impl<D: DiskManager> DirInner<D> {
    pub fn new(myself: InodePtr<D>, fs: Arc<FileSys<D>>) -> Self {
        Self { myself, fs: fs }
    }

    pub fn stat(&self) -> FileStat {
        let cache = self.fs.cache_manager().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = unsafe { &cache_guard.as_array::<Inode>()[self.myself.offset()] };

        FileStat::new(inode.size())
    }
}
