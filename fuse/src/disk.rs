use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use std::sync::Arc;

use lazy_static::lazy_static;
use spin::mutex::Mutex;

use crate::config::BLK_SIZE;

pub struct DiskManager {
    file: File,
}

lazy_static! {
    pub static ref DISK_MANAGER: Arc<Mutex<DiskManager>> =
        Arc::new(Mutex::new(DiskManager::new({
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(format!("{}", "fs.img"))
                .unwrap();
            f.set_len(33802 * 512).unwrap();
            f
        })));
}

impl DiskManager {
    pub fn read(&mut self, bid: usize, buf: &mut [u8]) {
        self.file
            .seek(SeekFrom::Start((bid * BLK_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            self.file.read(buf).unwrap(),
            BLK_SIZE,
            "Not a complete block!"
        );
    }

    pub fn write(&mut self, bid: usize, buf: &[u8]) {
        self.file
            .seek(SeekFrom::Start((bid * BLK_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            self.file.write(buf).unwrap(),
            BLK_SIZE,
            "Not a complete block!"
        );
    }
}

impl DiskManager {
    pub fn new(file: File) -> Self {
        Self { file }
    }
}
