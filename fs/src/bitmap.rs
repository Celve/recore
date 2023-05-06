use alloc::sync::Arc;

use crate::{cache::CacheManager, config::BLK_SIZE, disk::DiskManager};

pub struct BitMap<D: DiskManager> {
    /// The start block id of the bitmap.
    start_bid: usize,

    /// The number of blocks that the bitmap owns.
    len: usize,

    /// The number of availabe bits.
    available: usize,

    /// The manager that handles caches.
    cache_manager: Arc<CacheManager<D>>,
}

impl<D: DiskManager> BitMap<D> {
    pub fn alloc(&mut self) -> Option<usize> {
        if self.available > 0 {
            for bid in self.start_bid..self.start_bid + self.len {
                let cache = self.cache_manager.get(bid);
                let mut cache_guard = cache.lock();
                let data = cache_guard.as_array_mut::<u64>();

                let bytes_pair = data
                    .iter_mut()
                    .enumerate()
                    .find(|(_, &mut bytes)| bytes != u64::MAX);
                if let Some((bytes_id, bytes)) = bytes_pair {
                    let bit_id = bytes.trailing_ones() as usize;
                    *bytes |= 1 << bit_id;
                    self.available -= 1;
                    return Some((bid - self.start_bid) * BLK_SIZE + bytes_id * 64 + bit_id);
                }
            }
        }
        None
    }

    /// This function is safe, because if the bid is invalid, it will panic.
    pub fn dealloc(&mut self, bid: usize) {
        self.available += 1;
        assert!(self.clear(bid));
    }
}

impl<D: DiskManager> BitMap<D> {
    pub fn new(
        start_bid: usize,
        len: usize,
        available: usize,
        cache_manager: Arc<CacheManager<D>>,
    ) -> Self {
        Self {
            start_bid,
            len,
            available,
            cache_manager,
        }
    }

    /// Get the bit that indicates the flag of given inode id.
    pub fn get(&self, bid: usize) -> bool {
        let (blk, byte, bit) = self.locate(bid);

        let cache = self.cache_manager.get(self.start_bid + blk);
        let mut cache_guard = cache.lock();
        cache_guard.as_array_mut::<u64>()[byte] >> bit & 1 == 1
    }

    /// Set the bit that indicates the flag of given inode id; return the old value.
    pub fn set(&self, bid: usize) -> bool {
        let (blk, byte, bit) = self.locate(bid);

        let cache = self.cache_manager.get(self.start_bid + blk);
        let mut data = cache.lock();
        let data_guard = data.as_array_mut::<u64>();
        let old = data_guard[byte] >> bit & 1 == 1;
        data_guard[byte] |= 1 << bit;
        old
    }

    /// Clear the bit that indicates the flag of given inode id; return the old value.
    pub fn clear(&self, bid: usize) -> bool {
        let (blk, byte, bit) = self.locate(bid);

        let cache = self.cache_manager.get(self.start_bid + blk);
        let mut data = cache.lock();
        let data_guard = data.as_array_mut::<u64>();
        let old = data_guard[byte] >> bit & 1 == 1;
        data_guard[byte] &= !(1 << bit);
        old
    }

    fn locate(&self, iid: usize) -> (usize, usize, usize) {
        let blk = iid / BLK_SIZE;
        let bytes = iid % BLK_SIZE / 64;
        let bit = iid % BLK_SIZE % 64;

        if blk < self.start_bid || blk >= self.start_bid + self.len {
            panic!("Invalid inode id {}", iid);
        }

        (blk, bytes, bit)
    }
}
