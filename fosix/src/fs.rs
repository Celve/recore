use core::mem::size_of;

use bitflags::bitflags;

pub const DIR_ENTRY_NAME_LEN: usize = 28;

pub struct DirEntry {
    name: [u8; DIR_ENTRY_NAME_LEN],
    iid: u32,
}

pub struct FileStat {
    size: usize,
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const DIR = 1 << 8;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }

    pub struct SeekFlag: u8 {
        const SET = 0;
        const CUR = 1 << 0;
        const END = 1 << 1;
    }

    pub struct FilePerm: u8 {
        const READABLE = 1 << 0;
        const WRITEABLE = 1 << 1;
    }
}

impl DirEntry {
    pub fn empty() -> Self {
        Self {
            name: [0; DIR_ENTRY_NAME_LEN],
            iid: 0,
        }
    }

    pub fn new(name: &str, inode_id: usize) -> Self {
        let mut bytes = [0; DIR_ENTRY_NAME_LEN];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Self {
            name: bytes,
            iid: inode_id as u32,
        }
    }

    pub fn name(&self) -> &str {
        let len = (0..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }

    pub fn iid(&self) -> usize {
        self.iid as usize
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, size_of::<Self>()) }
    }
}

impl FileStat {
    pub fn empty() -> Self {
        Self { size: 0 }
    }

    pub fn new(size: usize) -> Self {
        Self { size }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }
}

impl From<OpenFlags> for FilePerm {
    fn from(flags: OpenFlags) -> Self {
        if flags.contains(OpenFlags::RDWR) {
            FilePerm::READABLE | FilePerm::WRITEABLE
        } else if flags.contains(OpenFlags::WRONLY) {
            FilePerm::WRITEABLE
        } else if flags.contains(OpenFlags::RDONLY) {
            FilePerm::READABLE
        } else {
            FilePerm::empty()
        }
    }
}
