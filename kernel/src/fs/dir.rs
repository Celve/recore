use core::mem::size_of;

use alloc::sync::Arc;
use alloc::{string::String, vec::Vec};
use spin::mutex::Mutex;

use super::{
    cache::CACHE_MANAGER,
    file::File,
    fuse::FUSE,
    inode::{Inode, InodePtr, InodeType},
};

use crate::config::INODE_PER_BLK;

const NAME_LENGTH: usize = 28;

pub struct Dir {
    inode_ptr: InodePtr,
}

pub struct DirEntry {
    name: [u8; NAME_LENGTH],
    inode_id: u32,
}

impl Dir {
    pub fn ls(&self) -> Vec<String> {
        let cache = CACHE_MANAGER.lock().get(self.inode_ptr.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.inode_ptr.offset()];

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
        if de.name() == name {
            let inode_ptr = InodePtr::new(de.inode_id as usize);

            let blk = CACHE_MANAGER.lock().get(inode_ptr.bid());
            let blk_guard = blk.lock();
            let inode = &blk_guard.as_array::<Inode>()[inode_ptr.offset()];
            if inode.is_dir() {
                return Some(Dir::new(inode_ptr));
            }
        }
        None
    }

    pub fn open(&self, name: &str) -> Option<File> {
        let de = self.get_de(name)?;
        if de.name() == name {
            let iid = de.inode_id as usize;
            let bid = iid / INODE_PER_BLK + FUSE.area_inode_start_bid();
            let offset = iid % INODE_PER_BLK;

            let blk = CACHE_MANAGER.lock().get(bid);
            let blk_guard = blk.lock();
            let inode = &blk_guard.as_array::<Inode>()[offset];
            if inode.is_file() {
                return Some(File::new(Arc::new(Mutex::new(Inode::from_existed(inode)))));
            }
        }
        None
    }

    pub fn mkdir(&self, name: &str) -> Result<(), ()> {
        self.create(name, InodeType::Directory)
    }

    pub fn touch(&self, name: &str) -> Result<(), ()> {
        self.create(name, InodeType::File)
    }

    fn get_de(&self, name: &str) -> Option<DirEntry> {
        let cache = CACHE_MANAGER.lock().get(self.inode_ptr.bid());
        let cache_guard = cache.lock();
        let inode = &cache_guard.as_array::<Inode>()[self.inode_ptr.offset()];

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
                *inode = Inode::empty(ty);
            }

            // modify outer inode
            {
                let cache = CACHE_MANAGER.lock().get(self.inode_ptr.bid());
                let mut cache_guard = cache.lock();
                let inode = &mut cache_guard.as_array_mut::<Inode>()[self.inode_ptr.offset()];
                inode.write(DirEntry::new(name, iid).as_bytes());
            }
            Ok(())
        }
    }
}

impl Dir {
    pub fn new(inode_ptr: InodePtr) -> Self {
        Self { inode_ptr }
    }
}

impl DirEntry {
    pub fn empty() -> Self {
        Self {
            name: [0; NAME_LENGTH],
            inode_id: 0,
        }
    }

    pub fn new(name: &str, inode_id: usize) -> Self {
        let mut bytes = [0; NAME_LENGTH];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Self {
            name: bytes,
            inode_id: inode_id as u32,
        }
    }

    pub fn name(&self) -> &str {
        let len = (0..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, size_of::<Self>()) }
    }
}
