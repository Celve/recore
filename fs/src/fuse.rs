use alloc::sync::Arc;
use spin::Spin;

use crate::{cache::CacheManager, disk::DiskManager};

use super::{
    bitmap::BitMap,
    dir::Dir,
    inode::{Inode, InodePtr},
    superblock::SuperBlock,
};

pub struct Fuse<D: DiskManager> {
    bitmap_inode: Spin<BitMap<D>>,
    area_inode_start_bid: usize,
    bitmap_dnode: Spin<BitMap<D>>,
    area_dnode_start_bid: usize,
    disk_manager: Arc<D>,
    cache_manager: Arc<CacheManager<D>>,
}

impl<D: DiskManager> Fuse<D> {
    pub fn root(self: &Arc<Self>) -> Dir<D> {
        Dir::new(InodePtr::new(0, self.clone()), self.clone())
    }
}

impl<D: DiskManager> Fuse<D> {
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

impl<D: DiskManager> Fuse<D> {
    pub fn new(super_block: SuperBlock, cache_manager: Arc<CacheManager<D>>) -> Self {
        let cache = cache_manager.get(0);
        let mut cache_guard = cache.lock();
        *cache_guard.as_any_mut::<SuperBlock>() = super_block;
        let bitmap_inode = BitMap::new(
            1,
            super_block.num_inode_bitmap_blks,
            super_block.num_inode,
            cache_manager.clone(),
        );
        let area_inode_start_bid = 1 + super_block.num_inode_bitmap_blks;
        let bitmap_dnode = BitMap::new(
            1 + super_block.num_inode_bitmap_blks + super_block.num_inode_area_blks,
            super_block.num_dnode_bitmap_blks,
            super_block.num_dnode,
            cache_manager.clone(),
        );
        let area_dnode_start_bid = 1
            + super_block.num_inode_bitmap_blks
            + super_block.num_inode_area_blks
            + super_block.num_dnode_bitmap_blks;

        Self {
            bitmap_inode: Spin::new(bitmap_inode),
            area_inode_start_bid,
            bitmap_dnode: Spin::new(bitmap_dnode),
            area_dnode_start_bid,
            disk_manager: cache_manager.disk_manager(),
            cache_manager,
        }
    }

    /// The allocation of root should be done at the start of the intialization.
    pub fn alloc_root(self: &Arc<Self>) {
        let iid = self.alloc_iid().unwrap();
        assert_eq!(iid, 0);
        let iptr = InodePtr::new(iid, self.clone());
        let blk = self.cache_manager.get(iptr.bid());
        let mut blk_guard = blk.lock();
        let inode = &mut blk_guard.as_array_mut::<Inode>()[iptr.offset()];
        *inode = Inode::empty_dir(iid, iid, self.clone());
    }

    // The caller has to make sure that there is a real fs image on the disk.
    pub unsafe fn from_existed(cache_manager: Arc<CacheManager<D>>) -> Self {
        let cache = cache_manager.get(0);
        let cache_guard = cache.lock();
        let super_block = unsafe { cache_guard.as_any::<SuperBlock>() };
        let bitmap_inode = BitMap::new(
            1,
            super_block.num_inode_bitmap_blks,
            super_block.num_inode,
            cache_manager.clone(),
        );
        let bitmap_dnode = BitMap::new(
            1 + super_block.num_inode_bitmap_blks + super_block.num_inode_area_blks,
            super_block.num_dnode_bitmap_blks,
            super_block.num_dnode,
            cache_manager.clone(),
        );
        Self {
            bitmap_inode: Spin::new(bitmap_inode),
            area_inode_start_bid: 1 + super_block.num_inode_bitmap_blks,
            bitmap_dnode: Spin::new(bitmap_dnode),
            area_dnode_start_bid: 1
                + super_block.num_inode_bitmap_blks
                + super_block.num_inode_area_blks
                + super_block.num_dnode_bitmap_blks,
            disk_manager: cache_manager.disk_manager(),
            cache_manager,
        }
    }

    pub fn area_inode_start_bid(&self) -> usize {
        self.area_inode_start_bid
    }

    pub fn area_dnode_start_bid(&self) -> usize {
        self.area_dnode_start_bid
    }

    pub fn super_block(&self) -> SuperBlock {
        let cache = self.cache_manager.get(0);
        let cache_guard = cache.lock();
        unsafe { cache_guard.as_any::<SuperBlock>().clone() }
    }

    pub fn disk_manager(&self) -> Arc<D> {
        self.disk_manager.clone()
    }

    pub fn cache_manager(&self) -> Arc<CacheManager<D>> {
        self.cache_manager.clone()
    }
}
