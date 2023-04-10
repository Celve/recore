pub mod disk;
pub mod fileable;
pub mod segment;

use crate::fs::disk::BlkDev;
use alloc::sync::Arc;
use fs::{cache::CacheManager, fuse::Fuse};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FUSE: Arc<Fuse<BlkDev>> = Arc::new(Fuse::from_existed(Arc::new(
        CacheManager::new(Arc::new(BlkDev::new()))
    )));
}
