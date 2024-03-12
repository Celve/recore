pub const CACHE_LEN: usize = 512;
pub const BLK_LEN: usize = 512;
pub const BLK_SIZE: usize = 4096;
pub const INODE_SIZE: usize = 32 * 4;
pub const DNODE_SIZE: usize = 32 * 16;
pub const INODE_PER_BLK: usize = BLK_LEN / INODE_SIZE;
