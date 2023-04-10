pub const CACHE_SIZE: usize = 512;
pub const BLK_SIZE: usize = 512;
pub const INODE_SIZE: usize = 32 * 4;
pub const DNODE_SIZE: usize = 32 * 16;
pub const INODE_PER_BLK: usize = BLK_SIZE / INODE_SIZE;
