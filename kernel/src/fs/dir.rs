use core::mem::size_of;

use alloc::{string::String, sync::Arc, vec::Vec};
use fosix::fs::{DirEntry, FileStat, OpenFlags};
use spin::mutex::{Mutex, MutexGuard};

use super::{
    cache::CACHE_MANAGER,
    file::File,
    fuse::FUSE,
    inode::{Inode, InodePtr, InodeType},
};

#[derive(Clone)]
pub struct Dir {
    inner: Arc<Mutex<DirInner>>,
}

pub struct DirInner {
    myself: InodePtr,
}

impl Dir {
    pub fn new(myself: InodePtr) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DirInner::new(myself))),
        }
    }

    pub fn lock(&self) -> MutexGuard<DirInner> {
        self.inner.lock()
    }
}

impl DirInner {
    pub fn ls(&self) -> Vec<String> {
        let cache = CACHE_MANAGER.lock().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        let num_de = inode.size() / size_of::<DirEntry>();
        let mut names = Vec::new();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(de.as_bytes_mut(), i * size_of::<DirEntry>());
            names.push(String::from(de.name()));
        }
        names
    }

    pub fn cd(&self, name: &str) -> Option<Dir> {
        let de = self.get_de(name)?;
        let inode_ptr = InodePtr::new(de.iid() as usize);

        let blk = CACHE_MANAGER.lock().get(inode_ptr.bid());
        let blk_guard = blk.lock();
        let inode = &blk_guard.as_array::<Inode>()[inode_ptr.offset()];
        if inode.is_dir() {
            Some(Dir::new(inode_ptr))
        } else {
            None
        }
    }

    pub fn open(&self, name: &str, flags: OpenFlags) -> Option<File> {
        let de = self.get_de(name);
        if let Some(de) = de {
            let inode_ptr = InodePtr::new(de.iid());

            let blk = CACHE_MANAGER.lock().get(inode_ptr.bid());
            let blk_guard = blk.lock();
            let inode = &blk_guard.as_array::<Inode>()[inode_ptr.offset()];
            if inode.is_file() {
                if flags.contains(OpenFlags::TRUNC) {
                    todo!()
                }
                return Some(File::new(inode_ptr, self.myself, flags.into()));
            }
        } else if flags.contains(OpenFlags::CREATE) {
            self.touch(name).unwrap();
            let de = self.get_de(name).unwrap();
            return Some(File::new(
                InodePtr::new(de.iid()),
                self.myself,
                flags.into(),
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
        let cache = CACHE_MANAGER.lock().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        let num_de = inode.size() / size_of::<DirEntry>();
        let mut des = Vec::new();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(de.as_bytes_mut(), i * size_of::<DirEntry>());
            des.push(de);
        }
        des
    }

    fn get_de(&self, name: &str) -> Option<DirEntry> {
        let cache = CACHE_MANAGER.lock().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        let num_de = inode.size() / size_of::<DirEntry>();
        assert_eq!(num_de * size_of::<DirEntry>(), inode.size());
        for i in 0..num_de {
            let mut de = DirEntry::empty();
            inode.read_at(de.as_bytes_mut(), i * size_of::<DirEntry>());
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
            let iid = FUSE.alloc_iid().unwrap();
            let inode_ptr = InodePtr::new(iid);

            // modify inner inode
            {
                let cache = CACHE_MANAGER.lock().get(inode_ptr.bid());
                let mut cache_guard = cache.lock();
                let inode = &mut cache_guard.as_array_mut::<Inode>()[inode_ptr.offset()];
                *inode = match ty {
                    InodeType::File => Inode::empty_file(),
                    InodeType::Directory => Inode::empty_dir(iid, self.myself.iid()),
                }
            }

            // modify outer inode
            {
                let cache = CACHE_MANAGER.lock().get(self.myself.bid());
                let mut cache_guard = cache.lock();
                let inode = &mut cache_guard.as_array_mut::<Inode>()[self.myself.offset()];
                inode.write_at_end(DirEntry::new(name, iid).as_bytes());
            }
            Ok(())
        }
    }
}

impl DirInner {
    pub fn new(myself: InodePtr) -> Self {
        Self { myself }
    }

    pub fn stat(&self) -> FileStat {
        let cache = CACHE_MANAGER.lock().get(self.myself.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.myself.offset()];

        FileStat::new(inode.size())
    }
}
