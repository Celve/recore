pub mod disk;
pub mod fileable;
pub mod segment;

use crate::fs::disk::BlockDevice;
use alloc::sync::Arc;
use fs::{cache::CacheManager, fuse::Fuse};
use lazy_static::lazy_static;

pub type Dir = fs::dir::Dir<BlockDevice>;
pub type File = fs::file::File<BlockDevice>;

lazy_static! {
    pub static ref FUSE: Arc<Fuse<BlockDevice>> = Arc::new(Fuse::from_existed(Arc::new(
        CacheManager::new(Arc::new(BlockDevice::new()))
    )));
}
