use core::{mem::size_of, num::NonZeroUsize, slice};

use alloc::{sync::Arc, vec::Vec};
use lru::LruCache;
use spin::Spin;

use crate::{
    config::{BLK_SIZE, CACHE_SIZE},
    disk::DiskManager,
};

pub struct CacheManager<D: DiskManager> {
    caches: Spin<LruCache<usize, Arc<Spin<Cache<D>>>>>,
    disk_manager: Arc<D>,
}

pub struct Cache<D: DiskManager> {
    data: Vec<u8>,
    bid: usize,
    dirt: bool,
    disk_manager: Arc<D>,
}

impl<D: DiskManager> CacheManager<D> {
    pub fn new(disk_manager: Arc<D>) -> Self {
        Self {
            caches: Spin::new(LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap())),
            disk_manager,
        }
    }

    pub fn get(&self, bid: usize) -> Arc<Spin<Cache<D>>> {
        let mut caches = self.caches.lock();
        if !caches.contains(&bid) {
            // caches is dropped here, because the creation of cache may be time-consuming
            drop(caches);
            let cache = Cache::new(bid, self.disk_manager.clone());
            caches = self.caches.lock();
            if !caches.contains(&bid) {
                caches.put(bid, Arc::new(Spin::new(cache)));
            }
        }
        caches.get(&bid).unwrap().clone()
    }

    pub fn clear(&self) {
        self.caches.lock().clear();
    }

    pub fn len(&self) -> usize {
        self.caches.lock().len()
    }

    pub fn disk_manager(&self) -> Arc<D> {
        self.disk_manager.clone()
    }
}

impl<D: DiskManager> Cache<D> {
    pub fn new(bid: usize, disk_manager: Arc<D>) -> Self {
        let mut data = vec![0; BLK_SIZE];
        disk_manager.read(bid, &mut data);
        Self {
            data,
            bid,
            dirt: false,
            disk_manager,
        }
    }

    /// Flush the block in the cache back to disk.
    pub fn sync(&mut self) {
        if self.dirt {
            self.disk_manager.write(self.bid, &mut self.data);
            self.dirt = false
        }
    }

    /// Caller must make sure that the data could be interpreted in this way.
    pub unsafe fn as_any<T>(&self) -> &T {
        unsafe { &*(self.data.as_ptr() as *const T) }
    }

    /// Caller must make sure that the data could be interpreted in this way.
    pub fn as_any_mut<T>(&mut self) -> &mut T {
        self.dirt = true;
        unsafe { &mut *(self.data.as_ptr() as *mut T) }
    }

    /// Caller must make sure that the data could be interpreted in this way.
    pub unsafe fn as_array<T>(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(
                self.data.as_ptr() as *mut T,
                self.data.len() / size_of::<T>(),
            )
        }
    }

    /// Caller must make sure that the data could be interpreted in this way.
    pub fn as_array_mut<T>(&mut self) -> &mut [T] {
        self.dirt = true;
        unsafe {
            slice::from_raw_parts_mut(
                self.data.as_ptr() as *mut T,
                self.data.len() / size_of::<T>(),
            )
        }
    }
}

impl<D: DiskManager> Drop for Cache<D> {
    fn drop(&mut self) {
        self.sync();
    }
}
