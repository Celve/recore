use core::cmp::{max, min};

use alloc::vec::Vec;
use fosix::fs::DirEntry;

use super::fuse::FUSE;
use crate::config::{DNODE_SIZE, INODE_PER_BLK, INODE_SIZE};

use super::cache::CACHE_MANAGER;

/// The length of dnode when it's seen as an array of u32.
const DNODE_LEN: usize = DNODE_SIZE / 4;

const DIRECT_INDEXING_LEN: usize = (INODE_SIZE - 4 * 4) / 4;
const DIRECT_INDEXING_MAX_LEN: usize = DIRECT_INDEXING_LEN;
const DIRECT_INDEXING_SIZE: usize = DIRECT_INDEXING_LEN * DNODE_SIZE;
const DIRECT_INDEXING_MAX_SIZE: usize = DIRECT_INDEXING_SIZE;

const INDIRECT1_INDEXING_LEN: usize = DNODE_LEN;
const INDIRECT1_INDEXING_MAX_LEN: usize = DIRECT_INDEXING_MAX_LEN + INDIRECT1_INDEXING_LEN;
const INDIRECT1_INDEXING_SIZE: usize = INDIRECT1_INDEXING_LEN * DNODE_SIZE;
const INDIRECT1_INDEXING_MAX_SIZE: usize = DIRECT_INDEXING_MAX_SIZE + INDIRECT1_INDEXING_SIZE;

const INDIRECT2_INDEXING_LEN: usize = DNODE_LEN * DNODE_LEN;
const INDIRECT2_INDEXING_MAX_LEN: usize = INDIRECT1_INDEXING_MAX_LEN + INDIRECT2_INDEXING_LEN;
const INDIRECT2_INDEXING_SIZE: usize = INDIRECT2_INDEXING_LEN * DNODE_SIZE;
const INDIRECT2_INDEXING_MAX_SIZE: usize = INDIRECT1_INDEXING_MAX_SIZE + INDIRECT2_INDEXING_SIZE;

pub struct Inode {
    /// The number of bytes the file that inode points to have.
    size: u32,

    /// The primary indirect mapping.
    indirect1: u32,

    /// The secondary indirect mapping.
    indirect2: u32,

    /// The type of inode.
    ty: InodeType,

    /// Direct mapping.
    directs: [u32; DIRECT_INDEXING_LEN],
}

#[derive(Clone, Copy)]
pub struct InodePtr {
    iid: usize,
    bid: usize,
    offset: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
}

impl Inode {
    /// Read from the given offset until the last. Return the actual bytes read.
    pub fn read_at(&self, buf: &mut [u8], offset: usize) -> usize {
        let blk_ids = self.find(offset, buf.len());
        let mut cnt = 0;
        for (i, blk_id) in blk_ids.iter().enumerate() {
            let blk = CACHE_MANAGER.lock().get(*blk_id as usize);
            let blk_guard = blk.lock();
            let bytes = blk_guard.as_array::<u8>();
            let start = if i == 0 { offset % DNODE_SIZE } else { 0 };
            let end = if i == blk_ids.len() - 1 {
                (offset + buf.len() - 1) % DNODE_SIZE + 1
            } else {
                DNODE_SIZE
            }; // exclusive
            for j in start..end {
                buf[cnt] = bytes[j];
                cnt += 1;
            }
        }
        cnt
    }

    /// Write to the given offset until the last. Return the actual bytes read.
    ///
    /// For the expansion of file, see `expand()`.
    pub fn write_at(&mut self, buf: &[u8], offset: usize) -> usize {
        let end = offset + buf.len();
        if end > self.size as usize {
            self.expand(end);
        }

        let blk_ids = self.find(offset, buf.len());
        let mut cnt = 0;
        for (i, blk_id) in blk_ids.iter().enumerate() {
            let blk = CACHE_MANAGER.lock().get(*blk_id as usize);
            let mut blk_guard = blk.lock();
            let bytes = blk_guard.as_array_mut::<u8>();
            let start = if i == 0 { offset % DNODE_SIZE } else { 0 };
            let end = if i == blk_ids.len() - 1 {
                (offset + buf.len() - 1) % DNODE_SIZE + 1
            } else {
                DNODE_SIZE
            }; // exclusive

            for j in start..end {
                bytes[j] = buf[cnt];
                cnt += 1;
            }
        }
        cnt
    }

    pub fn write_at_end(&mut self, buf: &[u8]) -> usize {
        self.write_at(buf, self.size as usize)
    }

    /// Adjust the file to the given size.
    ///
    /// If the new size is bigger than the current one, then it's an expansion; otherwise it's an shrinkage with extra bytes discarded.
    pub fn adjust(&mut self, new_size: usize) {
        let old_size = self.size as usize;
        if new_size > old_size {
            self.expand(new_size);
        } else if new_size < old_size {
            self.shrink(new_size);
        }
    }

    fn expand(&mut self, new_size: usize) {
        let mut start_blk_id = if self.size == 0 {
            0
        } else {
            (self.size as usize - 1) / DNODE_SIZE + 1
        }; // inclusive
        let end_blk_id = (new_size - 1) / DNODE_SIZE + 1; // exclusive
        self.size = new_size as u32;

        if start_blk_id < end_blk_id && start_blk_id < DIRECT_INDEXING_LEN {
            let end_blk_id = min(end_blk_id, DIRECT_INDEXING_LEN);
            for i in start_blk_id..end_blk_id {
                self.directs[i] = FUSE.alloc_bid().unwrap() as u32;
            }
        }
        start_blk_id = DIRECT_INDEXING_LEN;

        if start_blk_id < end_blk_id && start_blk_id < INDIRECT1_INDEXING_LEN {
            let offset = DIRECT_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT1_INDEXING_MAX_LEN) - offset;
            if start_blk_id == 0 {
                self.indirect1 = FUSE.alloc_bid().unwrap() as u32;
            }
            self.set_primary(start_blk_id, end_blk_id);
        }
        start_blk_id = INDIRECT1_INDEXING_MAX_LEN;

        if start_blk_id < end_blk_id {
            let offset = INDIRECT1_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT2_INDEXING_MAX_LEN) - offset;
            if start_blk_id == 0 {
                self.indirect2 = FUSE.alloc_bid().unwrap() as u32;
            }
            self.set_secondary(start_blk_id, end_blk_id);
        }
    }

    fn shrink(&mut self, new_size: usize) {
        todo!();
    }

    pub fn find(&self, start: usize, len: usize) -> Vec<u32> {
        let end = min(start + len, self.size as usize); // exclusive
        let mut start_blk_id = start / DNODE_SIZE;
        let end_blk_id = (max(end, 1) - 1) / DNODE_SIZE + 1; // exclusive
        let mut res = Vec::new();

        if start_blk_id < end_blk_id && start_blk_id < DIRECT_INDEXING_LEN {
            let end_blk_id = min(end_blk_id, DIRECT_INDEXING_LEN);
            res.extend(self.directs[start_blk_id..end_blk_id].iter());
        }
        start_blk_id = DIRECT_INDEXING_MAX_LEN;

        if start_blk_id < end_blk_id && start_blk_id < INDIRECT1_INDEXING_MAX_LEN {
            let offset = DIRECT_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT1_INDEXING_MAX_LEN) - offset;
            res.append(&mut self.get_primary(start_blk_id, end_blk_id));
        }
        start_blk_id = INDIRECT1_INDEXING_MAX_LEN;

        if start_blk_id < end_blk_id {
            let offset = INDIRECT1_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT2_INDEXING_MAX_LEN) - offset;
            res.append(&mut self.get_secondary(start_blk_id, end_blk_id));
        }
        assert!(res.iter().all(|id| *id != 0));
        res
    }

    pub fn get_primary(&self, start_blk_id: usize, end_blk_id: usize) -> Vec<u32> {
        let pri = CACHE_MANAGER.lock().get(self.indirect1 as usize);
        let mut pri_guard = pri.lock();
        let blk_ids = pri_guard.as_array_mut::<u32>();

        let mut res = Vec::new();
        res.extend(blk_ids[start_blk_id..end_blk_id].iter());
        res
    }

    pub fn get_secondary(&self, start_blk_id: usize, end_blk_id: usize) -> Vec<u32> {
        let sec = CACHE_MANAGER.lock().get(self.indirect2 as usize);
        let mut sec_guard = sec.lock();
        let pri_ids = sec_guard.as_array_mut::<u32>();
        let start_pri_offset = start_blk_id / DNODE_LEN;
        let end_pri_offset = (end_blk_id - 1) / DNODE_LEN + 1;
        let mut res = Vec::new();
        for i in start_pri_offset..end_pri_offset {
            let pri = CACHE_MANAGER.lock().get(pri_ids[i] as usize);
            let mut pri_guard = pri.lock();
            let blk_ids = pri_guard.as_array_mut::<u32>();
            let start_blk_id = if i == start_pri_offset {
                start_blk_id % DNODE_LEN
            } else {
                0
            };
            let end_blk_id = if i == end_pri_offset - 1 {
                (end_blk_id - 1) % DNODE_LEN + 1
            } else {
                DNODE_LEN
            };
            res.extend(blk_ids[start_blk_id..end_blk_id].iter());
        }
        res
    }

    pub fn set_primary(&self, start_blk_id: usize, end_blk_id: usize) {
        let pri = CACHE_MANAGER.lock().get(self.indirect1 as usize);
        let mut pri_guard = pri.lock();
        let blk_ids = pri_guard.as_array_mut::<u32>();

        blk_ids[start_blk_id..end_blk_id]
            .iter_mut()
            .for_each(|blk_id| *blk_id = FUSE.alloc_bid().unwrap() as u32);
    }

    pub fn set_secondary(&self, start_blk_id: usize, end_blk_id: usize) {
        let sec = CACHE_MANAGER.lock().get(self.indirect2 as usize);
        let mut sec_guard = sec.lock();
        let pri_ids = sec_guard.as_array_mut::<u32>();
        let start_pri_offset = start_blk_id / DNODE_LEN;
        let end_pri_offset = (end_blk_id - 1) / DNODE_LEN + 1;
        for i in start_pri_offset..end_pri_offset {
            let pri_id = FUSE.alloc_bid().unwrap();
            pri_ids[i] = pri_id as u32;
            let pri = CACHE_MANAGER.lock().get(pri_id);
            let mut pri_guard = pri.lock();
            let blk_ids = pri_guard.as_array_mut::<u32>();
            let start_blk_id = if i == start_pri_offset {
                start_blk_id % DNODE_LEN
            } else {
                0
            };
            let end_blk_id = if i == end_pri_offset - 1 {
                (end_blk_id - 1) % DNODE_LEN + 1
            } else {
                DNODE_LEN
            };
            blk_ids[start_blk_id..end_blk_id]
                .iter_mut()
                .for_each(|blk_id| {
                    *blk_id = FUSE.alloc_bid().unwrap() as u32;
                });
        }
    }
}

impl Inode {
    fn empty(ty: InodeType) -> Self {
        Self {
            size: 0,
            indirect1: 0,
            indirect2: 0,
            ty,
            directs: [0; DIRECT_INDEXING_LEN],
        }
    }

    pub fn empty_file() -> Self {
        Self::empty(InodeType::File)
    }

    pub fn empty_dir(myself: usize, parent: usize) -> Self {
        let mut inode = Self::empty(InodeType::Directory);
        inode.write_at_end(DirEntry::new(".", myself).as_bytes());
        inode.write_at_end(DirEntry::new("..", parent).as_bytes());
        inode
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn is_file(&self) -> bool {
        self.ty == InodeType::File
    }

    pub fn is_dir(&self) -> bool {
        self.ty == InodeType::Directory
    }
}

impl InodePtr {
    pub fn new(iid: usize) -> Self {
        Self {
            iid,
            bid: iid / INODE_PER_BLK + FUSE.area_inode_start_bid(),
            offset: iid % INODE_PER_BLK,
        }
    }

    pub fn iid(&self) -> usize {
        self.iid
    }

    pub fn bid(&self) -> usize {
        self.bid
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}
