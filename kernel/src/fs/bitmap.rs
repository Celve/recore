use crate::config::BLK_SIZE;

use super::cache::CACHE_MANAGER;

pub struct BitMap {
    /// The start block id of the bitmap.
    start_bid: usize,

    /// The number of blocks that the bitmap owns.
    len: usize,

    /// The number of availabe bits.
    available: usize,
}

impl BitMap {
    pub fn alloc(&mut self) -> Option<usize> {
        if self.available > 0 {
            for bid in self.start_bid..self.start_bid + self.len {
                let cache = CACHE_MANAGER.lock().get(bid);
                let mut cache_guard = cache.try_lock().unwrap();
                let data = cache_guard.as_array_mut::<u64>();
                let bytes_pair = data
                    .iter_mut()
                    .enumerate()
                    .find(|(_, &mut bytes)| bytes != u64::MAX);
                if let Some((bytes_id, bytes)) = bytes_pair {
                    let bit_id = bytes.trailing_ones() as usize;
                    *bytes |= 1 << bit_id;
                    self.available -= 1;
                    return Some((bid - self.start_bid) * BLK_SIZE * 8 + bytes_id * 64 + bit_id);
                }
            }
        }
        None
    }

    pub fn dealloc(&mut self, bid: usize) {
        self.available += 1;
        assert!(self.clear(bid));
    }
}

impl BitMap {
    pub fn new(start_bid: usize, len: usize, available: usize) -> Self {
        Self {
            start_bid,
            len,
            available,
        }
    }

    /// Get the bit that indicates the flag of given inode id.
    pub fn get(&self, bid: usize) -> bool {
        let (blk, byte, bit) = Self::locate(bid);

        let cache = CACHE_MANAGER.lock().get(self.start_bid + blk);
        let mut cache_guard = cache.lock();
        cache_guard.as_array_mut::<u64>()[byte] >> bit & 1 == 1
    }

    /// Set the bit that indicates the flag of given inode id; return the old value.
    pub fn set(&self, bid: usize) -> bool {
        let (blk, byte, bit) = Self::locate(bid);

        let cache = CACHE_MANAGER.lock().get(self.start_bid + blk);
        let mut data = cache.lock();
        let data_guard = data.as_array_mut::<u64>();
        let old = data_guard[byte] >> bit & 1 == 1;
        data_guard[byte] |= 1 << bit;
        old
    }

    /// Clear the bit that indicates the flag of given inode id; return the old value.
    pub fn clear(&self, bid: usize) -> bool {
        let (blk, byte, bit) = Self::locate(bid);

        let cache = CACHE_MANAGER.lock().get(self.start_bid + blk);
        let mut data = cache.lock();
        let data_guard = data.as_array_mut::<u64>();
        let old = data_guard[byte] >> bit & 1 == 1;
        data_guard[byte] &= !(1 << bit);
        old
    }

    fn locate(iid: usize) -> (usize, usize, usize) {
        let blk = iid / (BLK_SIZE * 8);
        let bytes = iid % (BLK_SIZE * 8) / 64;
        let bit = iid % (BLK_SIZE * 8) % 64;
        (blk, bytes, bit)
    }
}
