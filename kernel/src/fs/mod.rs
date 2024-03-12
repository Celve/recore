pub mod fileable;
pub mod segment;

use crate::drivers::blockdev::BlkDev;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use simplefs::{cache::CacheManager, fs::FileSys};

lazy_static! {
    pub static ref FS: Arc<FileSys<BlkDev>> = unsafe {
        Arc::new(FileSys::from_existed(Arc::new(CacheManager::new(
            Arc::new(BlkDev::new()),
        ))))
    };
}
