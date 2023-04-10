use crate::config::{BLK_SIZE, INODE_PER_BLK};

#[derive(Clone, Copy)]
pub struct SuperBlock {
    pub magic: usize,
    pub num_blks: usize,
    pub num_inode: usize,
    pub num_inode_bitmap_blks: usize,
    pub num_inode_area_blks: usize,
    pub num_dnode: usize,
    pub num_dnode_bitmap_blks: usize,
    pub num_dnode_area_blks: usize,
}

impl SuperBlock {
    pub fn new(num_inode: usize, num_dnode: usize) -> Self {
        let magic = 7;
        let num_inode_bitmap_blks = (num_inode / 8 - 1) / BLK_SIZE + 1;
        let num_inode_area_blks = (num_inode - 1) / INODE_PER_BLK + 1; // because every block could hold up to 3 inodes
        let num_dnode_bitmap_blks = (num_dnode / 8 - 1) / BLK_SIZE + 1;
        let num_dnode_area_blks = num_dnode;
        let num_blks = 1
            + num_inode_bitmap_blks
            + num_inode_area_blks
            + num_dnode_bitmap_blks
            + num_dnode_area_blks;
        Self {
            magic,
            num_blks,
            num_inode,
            num_inode_bitmap_blks,
            num_inode_area_blks,
            num_dnode,
            num_dnode_bitmap_blks,
            num_dnode_area_blks,
        }
    }
}
