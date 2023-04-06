use lazy_static::lazy_static;
use spin::mutex::Mutex;

use super::{
    bitmap::BitMap,
    cache::CACHE_MANAGER,
    dir::Dir,
    inode::{Inode, InodePtr},
    superblock::SuperBlock,
};

pub struct Fuse {
    bitmap_inode: Mutex<BitMap>,
    area_inode_start_bid: usize,
    bitmap_dnode: Mutex<BitMap>,
    area_dnode_start_bid: usize,
}

lazy_static! {
    pub static ref FUSE: Fuse = Fuse::from_existed();
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

    pub fn dealloc_bid(&self, bid: usize) {
        self.bitmap_dnode
            .lock()
            .dealloc(bid - self.area_dnode_start_bid)
    }

    pub fn alloc_iid(&self) -> Option<usize> {
        self.bitmap_inode.lock().alloc()
    }

    pub fn dealloc_iid(&self, iid: usize) {
        self.bitmap_inode.lock().dealloc(iid)
    }
}

impl Fuse {
    pub fn new(super_block: SuperBlock) -> Self {
        let cache = CACHE_MANAGER.lock().get(0);
        let mut cache_guard = cache.lock();
        *cache_guard.as_any_mut::<SuperBlock>() = super_block;
        let bitmap_inode = BitMap::new(1, super_block.num_inode_bitmap_blks, super_block.num_inode);
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

        Self {
            bitmap_inode: Mutex::new(bitmap_inode),
            area_inode_start_bid,
            bitmap_dnode: Mutex::new(bitmap_dnode),
            area_dnode_start_bid,
        }
    }

    pub fn alloc_root(&self) {
        let iid = self.alloc_iid().unwrap();
        assert_eq!(iid, 0);
        let iptr = InodePtr::new(iid);
        let blk = CACHE_MANAGER.lock().get(iptr.bid());
        let mut blk_guard = blk.lock();
        let inode = &mut blk_guard.as_array_mut::<Inode>()[iptr.offset()];
        *inode = Inode::empty_dir(iid, iid);
    }

    pub fn from_existed() -> Self {
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
