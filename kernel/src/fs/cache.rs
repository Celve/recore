use core::{mem::size_of, num::NonZeroUsize, slice};

use alloc::sync::Arc;
use lazy_static::lazy_static;
use lru::LruCache;
use spin::mutex::Mutex;

use crate::config::{BLK_SIZE, CACHE_SIZE};

use super::disk::DISK_MANAGER;

pub struct CacheManager {
    pub caches: LruCache<usize, Arc<Mutex<Cache>>>,
}

pub struct Cache {
    data: [u8; BLK_SIZE],
    bid: usize,
    pub dirt: bool,
}

lazy_static! {
    pub static ref CACHE_MANAGER: Mutex<CacheManager> = Mutex::new(CacheManager::new());
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            caches: LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap()),
        }
    }

    pub fn get(&mut self, bid: usize) -> Arc<Mutex<Cache>> {
        if !self.caches.contains(&bid) {
            self.caches.put(bid, Arc::new(Mutex::new(Cache::new(bid))));
        }
        self.caches.get(&bid).unwrap().clone()
    }

    pub fn clear(&mut self) {
        self.caches.clear();
    }

    pub fn len(&self) -> usize {
        self.caches.len()
    }
}

impl Cache {
    pub fn new(bid: usize) -> Self {
        let mut data = [0; BLK_SIZE];
        DISK_MANAGER.lock().read(bid, &mut data);
        Self {
            data,
            bid,
            dirt: false,
        }
    }

    pub fn sync(&mut self) {
        if self.dirt {
            DISK_MANAGER.lock().write(self.bid, &mut self.data);
            self.dirt = false
        }
    }

    pub fn as_any<T>(&self) -> &T {
        unsafe { &*(self.data.as_ptr() as *const T) }
    }

    pub fn as_any_mut<T>(&mut self) -> &mut T {
        self.dirt = true;
        unsafe { &mut *(self.data.as_ptr() as *mut T) }
    }

    pub fn as_array<T>(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(
                self.data.as_ptr() as *mut T,
                self.data.len() / size_of::<T>(),
            )
        }
    }

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

impl Drop for Cache {
    fn drop(&mut self) {
        self.sync();
    }
}
