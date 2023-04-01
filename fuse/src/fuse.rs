use lazy_static::lazy_static;
use spin::mutex::Mutex;
use std::sync::Arc;

use super::{
    bitmap::BitMap,
    cache::CACHE_MANAGER,
    dir::Dir,
    disk::{DiskManager, DISK_MANAGER},
    inode::{Inode, InodePtr, InodeType},
    superblock::SuperBlock,
};

use crate::config::{INODE_PER_BLK, INODE_SIZE};

pub struct Fuse {
    disk_manager: Arc<Mutex<DiskManager>>,
    bitmap_inode: Mutex<BitMap>,
    area_inode_start_bid: usize,
    bitmap_dnode: Mutex<BitMap>,
    area_dnode_start_bid: usize,
}

lazy_static! {
    pub static ref FUSE: Fuse = Fuse::new(
        SuperBlock::new(4096, 32768),
        DISK_MANAGER.clone(),
    );
    // pub static ref FUSE: Arc<Mutex<Fuse>> = Arc::new(Mutex::new(Fuse::from_existed(
    //     DISK_MANAGER.clone(),
    // )));
}

impl Fuse {
    pub fn root(&self) -> Dir {
        Dir::new(InodePtr::new(0))
    }
}

impl Fuse {
    pub fn alloc_bid(&self) -> Option<usize> {
        Some(self.bitmap_dnode.lock().alloc()? + self.area_dnode_start_bid)
    }

    pub fn alloc_iid(&self) -> Option<usize> {
        self.bitmap_inode.lock().alloc()
    }
}

impl Fuse {
    pub fn new(super_block: SuperBlock, disk_manager: Arc<Mutex<DiskManager>>) -> Self {
        let cache = CACHE_MANAGER.lock().get(0);
        let mut cache_guard = cache.lock();
        *cache_guard.as_any_mut::<SuperBlock>() = super_block;
        let mut bitmap_inode =
            BitMap::new(1, super_block.num_inode_bitmap_blks, super_block.num_inode);
        let area_inode_start_bid = 1 + super_block.num_inode_bitmap_blks;
        let bitmap_dnode = BitMap::new(
            1 + super_block.num_inode_bitmap_blks + super_block.num_inode_area_blks,
            super_block.num_dnode_bitmap_blks,
            super_block.num_dnode,
        );
        let area_dnode_start_bid = 1
            + super_block.num_inode_bitmap_blks
            + super_block.num_inode_area_blks
            + super_block.num_dnode_bitmap_blks;

        // alloc root
        let iid = bitmap_inode.alloc().unwrap();
        assert_eq!(iid, 0);
        let bid = area_inode_start_bid + iid / INODE_PER_BLK;
        let offset = iid % INODE_PER_BLK * INODE_SIZE;
        let blk = CACHE_MANAGER.lock().get(bid);
        let mut blk_guard = blk.lock();
        let inode = &mut blk_guard.as_array_mut::<Inode>()[offset];
        *inode = Inode::empty(InodeType::Directory);

        Self {
            disk_manager,
            bitmap_inode: Mutex::new(bitmap_inode),
            area_inode_start_bid,
            bitmap_dnode: Mutex::new(bitmap_dnode),
            area_dnode_start_bid,
        }
    }

    pub fn from_existed(disk_manager: Arc<Mutex<DiskManager>>) -> Self {
        let cache = CACHE_MANAGER.lock().get(0);
        let cache_guard = cache.lock();
        let super_block = cache_guard.as_any::<SuperBlock>();
        let bitmap_inode = BitMap::new(1, super_block.num_inode_bitmap_blks, super_block.num_inode);
        let bitmap_dnode = BitMap::new(
            1 + super_block.num_inode_bitmap_blks + super_block.num_inode_area_blks,
            super_block.num_dnode_bitmap_blks,
            super_block.num_dnode,
        );
        Self {
            disk_manager,
            bitmap_inode: Mutex::new(bitmap_inode),
            area_inode_start_bid: 1 + super_block.num_inode_bitmap_blks,
            bitmap_dnode: Mutex::new(bitmap_dnode),
            area_dnode_start_bid: 1
                + super_block.num_inode_bitmap_blks
                + super_block.num_inode_area_blks
                + super_block.num_dnode_bitmap_blks,
        }
    }

    pub fn area_inode_start_bid(&self) -> usize {
        self.area_inode_start_bid
    }

    pub fn area_dnode_start_bid(&self) -> usize {
        self.area_dnode_start_bid
    }

    pub fn super_block(&self) -> SuperBlock {
        let cache = CACHE_MANAGER.lock().get(0);
        let cache_guard = cache.lock();
        cache_guard.as_any::<SuperBlock>().clone()
    }
}
