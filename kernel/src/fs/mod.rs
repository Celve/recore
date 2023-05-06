pub mod fileable;
pub mod segment;

use crate::drivers::blockdev::BlkDev;
use alloc::sync::Arc;
use fs::{cache::CacheManager, fuse::Fuse};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FUSE: Arc<Fuse<BlkDev>> = unsafe {
        Arc::new(Fuse::from_existed(Arc::new(CacheManager::new(Arc::new(
            BlkDev::new(),
        )))))
    };
}
