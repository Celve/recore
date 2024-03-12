use core::cmp::{max, min};

use alloc::{sync::Arc, vec::Vec};
use fosix::fs::DirEntry;

use crate::{
    config::{DNODE_SIZE, INODE_PER_BLK, INODE_SIZE},
    disk::DiskManager,
    fs::FileSys,
};

/// The length of dnode when it's seen as an array of u32.
const DNODE_LEN: usize = DNODE_SIZE / 4;

const DIRECT_INDEXING_LEN: usize = (INODE_SIZE - 4 * 4) / 4;
const DIRECT_INDEXING_MAX_LEN: usize = DIRECT_INDEXING_LEN;

const INDIRECT1_INDEXING_LEN: usize = DNODE_LEN;
const INDIRECT1_INDEXING_MAX_LEN: usize = DIRECT_INDEXING_MAX_LEN + INDIRECT1_INDEXING_LEN;

const INDIRECT2_INDEXING_LEN: usize = DNODE_LEN * DNODE_LEN;
const INDIRECT2_INDEXING_MAX_LEN: usize = INDIRECT1_INDEXING_MAX_LEN + INDIRECT2_INDEXING_LEN;

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

pub struct InodePtr<D: DiskManager> {
    iid: usize,
    bid: usize,
    offset: usize,
    fs: Arc<FileSys<D>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
}

impl<D: DiskManager> Clone for InodePtr<D> {
    fn clone(&self) -> Self {
        Self {
            iid: self.iid,
            bid: self.bid,
            offset: self.offset,
            fs: self.fs.clone(),
        }
    }
}

impl Inode {
    /// Read from the given offset until the last. Return the actual bytes read.
    pub fn read_at<D: DiskManager>(
        &self,
        buf: &mut [u8],
        offset: usize,
        fs: Arc<FileSys<D>>,
    ) -> usize {
        let blk_ids = self.find(offset, buf.len(), fs.clone());
        let mut cnt = 0;
        for (i, blk_id) in blk_ids.iter().enumerate() {
            let blk = fs.cache_manager().get(*blk_id as usize);
            let blk_guard = blk.lock();
            let bytes = unsafe { blk_guard.as_array::<u8>() };
            let start = if i == 0 { offset % DNODE_SIZE } else { 0 };
            let end = if i == blk_ids.len() - 1 {
                (min(offset + buf.len(), self.size()) - 1) % DNODE_SIZE + 1
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
    pub fn write_at<D: DiskManager>(
        &mut self,
        buf: &[u8],
        offset: usize,
        fs: Arc<FileSys<D>>,
    ) -> usize {
        let end = offset + buf.len();
        if end > self.size as usize {
            self.expand(end, fs.clone());
        }

        let blk_ids = self.find(offset, buf.len(), fs.clone());
        let mut cnt = 0;
        for (i, blk_id) in blk_ids.iter().enumerate() {
            let blk = fs.cache_manager().get(*blk_id as usize);
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

    pub fn write_at_end<D: DiskManager>(&mut self, buf: &[u8], fs: Arc<FileSys<D>>) -> usize {
        self.write_at(buf, self.size as usize, fs)
    }

    pub fn trunc<D: DiskManager>(&mut self, fs: Arc<FileSys<D>>) -> usize {
        let old_size = self.size as usize;
        self.adjust(0, fs);
        old_size
    }

    /// Adjust the file to the given size.
    ///
    /// If the new size is bigger than the current one, then it's an expansion; otherwise it's an shrinkage with extra bytes discarded.
    pub fn adjust<D: DiskManager>(&mut self, new_size: usize, fs: Arc<FileSys<D>>) {
        let old_size = self.size as usize;
        if new_size > old_size {
            self.expand(new_size, fs);
        } else if new_size < old_size {
            self.shrink(new_size, fs);
        }
    }

    fn expand<D: DiskManager>(&mut self, new_size: usize, fs: Arc<FileSys<D>>) {
        assert!(new_size > self.size());
        let start_blk_id = if self.size == 0 {
            0
        } else {
            (self.size as usize - 1) / DNODE_SIZE + 1
        }; // inclusive
        let end_blk_id = (new_size - 1) / DNODE_SIZE + 1; // exclusive
        self.size = new_size as u32;
        self.set(start_blk_id, end_blk_id, true, fs);
    }

    fn shrink<D: DiskManager>(&mut self, new_size: usize, fs: Arc<FileSys<D>>) {
        assert!(new_size < self.size());
        let start_blk_id = if new_size == 0 {
            0
        } else {
            (new_size - 1) / DNODE_SIZE + 1
        }; // inclusive
        let end_blk_id = (self.size as usize - 1) / DNODE_SIZE + 1; // exclusive
        self.size = new_size as u32;
        self.set(start_blk_id, end_blk_id, false, fs);
    }

    fn set<D: DiskManager>(
        &mut self,
        mut start_blk_id: usize,
        end_blk_id: usize,
        flag: bool,
        fs: Arc<FileSys<D>>,
    ) {
        if start_blk_id < end_blk_id && start_blk_id < DIRECT_INDEXING_MAX_LEN {
            let end_blk_id = min(end_blk_id, DIRECT_INDEXING_MAX_LEN);
            for i in start_blk_id..end_blk_id {
                if flag {
                    self.directs[i] = fs.alloc_bid().unwrap() as u32;
                } else {
                    fs.dealloc_bid(self.directs[i] as usize);
                }
            }
        }
        start_blk_id = max(start_blk_id, DIRECT_INDEXING_MAX_LEN);

        if start_blk_id < end_blk_id && start_blk_id < INDIRECT1_INDEXING_MAX_LEN {
            let offset = DIRECT_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT1_INDEXING_MAX_LEN) - offset;
            if start_blk_id == 0 && flag {
                self.indirect1 = fs.alloc_bid().unwrap() as u32;
            }
            self.set_primary(start_blk_id, end_blk_id, flag, fs.clone());
            if start_blk_id == 0 && !flag {
                fs.dealloc_bid(self.indirect1 as usize);
            }
        }
        start_blk_id = max(start_blk_id, INDIRECT1_INDEXING_MAX_LEN);

        if start_blk_id < end_blk_id {
            let offset = INDIRECT1_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT2_INDEXING_MAX_LEN) - offset;
            if start_blk_id == 0 && flag {
                self.indirect2 = fs.alloc_bid().unwrap() as u32;
            }
            self.set_secondary(start_blk_id, end_blk_id, flag, fs.clone());
            if start_blk_id == 0 && !flag {
                fs.dealloc_bid(self.indirect2 as usize);
            }
        }
    }

    pub fn find<D: DiskManager>(&self, start: usize, len: usize, fs: Arc<FileSys<D>>) -> Vec<u32> {
        let end = min(start + len, self.size as usize); // exclusive
        let mut start_blk_id = start / DNODE_SIZE;
        let end_blk_id = if end == 0 {
            0
        } else {
            (end - 1) / DNODE_SIZE + 1
        }; // exclusive
        let mut res = Vec::new();

        if start_blk_id < end_blk_id && start_blk_id < DIRECT_INDEXING_MAX_LEN {
            let end_blk_id = min(end_blk_id, DIRECT_INDEXING_MAX_LEN);
            res.extend(self.directs[start_blk_id..end_blk_id].iter());
        }
        start_blk_id = max(start_blk_id, DIRECT_INDEXING_MAX_LEN);

        if start_blk_id < end_blk_id && start_blk_id < INDIRECT1_INDEXING_MAX_LEN {
            let offset = DIRECT_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT1_INDEXING_MAX_LEN) - offset;
            res.append(&mut self.get_primary(start_blk_id, end_blk_id, fs.clone()));
        }
        start_blk_id = max(start_blk_id, INDIRECT1_INDEXING_MAX_LEN);

        if start_blk_id < end_blk_id {
            let offset = INDIRECT1_INDEXING_MAX_LEN;
            let start_blk_id = start_blk_id - offset;
            let end_blk_id = min(end_blk_id, INDIRECT2_INDEXING_MAX_LEN) - offset;
            res.append(&mut self.get_secondary(start_blk_id, end_blk_id, fs.clone()));
        }
        assert!(res.iter().all(|id| *id != 0));
        res
    }

    pub fn get_primary<D: DiskManager>(
        &self,
        start_blk_id: usize,
        end_blk_id: usize,
        fs: Arc<FileSys<D>>,
    ) -> Vec<u32> {
        let pri = fs.cache_manager().get(self.indirect1 as usize);
        let mut pri_guard = pri.lock();
        let blk_ids = pri_guard.as_array_mut::<u32>();

        let mut res = Vec::new();
        res.extend(blk_ids[start_blk_id..end_blk_id].iter());
        res
    }

    pub fn get_secondary<D: DiskManager>(
        &self,
        start_blk_id: usize,
        end_blk_id: usize,
        fs: Arc<FileSys<D>>,
    ) -> Vec<u32> {
        let sec = fs.cache_manager().get(self.indirect2 as usize);
        let mut sec_guard = sec.lock();
        let pri_ids = sec_guard.as_array_mut::<u32>();
        let start_pri_offset = start_blk_id / DNODE_LEN;
        let end_pri_offset = (end_blk_id - 1) / DNODE_LEN + 1;
        let mut res = Vec::new();
        for i in start_pri_offset..end_pri_offset {
            let pri = fs.cache_manager().get(pri_ids[i] as usize);
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

    pub fn set_primary<D: DiskManager>(
        &self,
        start_blk_id: usize,
        end_blk_id: usize,
        flag: bool,
        fs: Arc<FileSys<D>>,
    ) {
        let pri = fs.cache_manager().get(self.indirect1 as usize);
        let mut pri_guard = pri.lock();
        let blk_ids = pri_guard.as_array_mut::<u32>();

        blk_ids[start_blk_id..end_blk_id]
            .iter_mut()
            .for_each(|blk_id| {
                if flag {
                    *blk_id = fs.alloc_bid().unwrap() as u32;
                } else {
                    fs.dealloc_bid(*blk_id as usize);
                }
            });
    }

    pub fn set_secondary<D: DiskManager>(
        &self,
        start_blk_id: usize,
        end_blk_id: usize,
        flag: bool,
        fs: Arc<FileSys<D>>,
    ) {
        let sec = fs.cache_manager().get(self.indirect2 as usize);
        let mut sec_guard = sec.lock();
        let pri_ids = sec_guard.as_array_mut::<u32>();
        let start_pri_offset = start_blk_id / DNODE_LEN;
        let end_pri_offset = (end_blk_id - 1) / DNODE_LEN + 1;
        for i in start_pri_offset..end_pri_offset {
            let pri_id = if flag {
                let bid = fs.alloc_bid().unwrap();
                pri_ids[i] = bid as u32;
                bid
            } else {
                pri_ids[i] as usize
            };

            let pri = fs.cache_manager().get(pri_id);
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
                    if flag {
                        *blk_id = fs.alloc_bid().unwrap() as u32;
                    } else {
                        fs.dealloc_bid(*blk_id as usize);
                    }
                });

            if !flag {
                fs.dealloc_bid(pri_id);
            }
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

    pub fn empty_dir<D: DiskManager>(myself: usize, parent: usize, fs: Arc<FileSys<D>>) -> Self {
        let mut inode = Self::empty(InodeType::Directory);
        inode.write_at_end(DirEntry::new(".", myself).as_bytes(), fs.clone());
        inode.write_at_end(DirEntry::new("..", parent).as_bytes(), fs.clone());
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

impl<D: DiskManager> InodePtr<D> {
    pub fn new(iid: usize, fs: Arc<FileSys<D>>) -> Self {
        Self {
            iid,
            bid: iid / INODE_PER_BLK + fs.area_inode_start_bid(),
            offset: iid % INODE_PER_BLK,
            fs,
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
