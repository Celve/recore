use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use simplefs::disk::DiskManager;
use spin::Spin;

use simplefs::config::BLK_LEN;

pub struct FileDev {
    file: Spin<File>,
}

impl DiskManager for FileDev {
    fn read(&self, bid: usize, buf: &mut [u8]) {
        self.file
            .lock()
            .seek(SeekFrom::Start((bid * BLK_LEN) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            self.file.lock().read(buf).unwrap(),
            BLK_LEN,
            "Not a complete block!"
        );
    }

    fn write(&self, bid: usize, buf: &[u8]) {
        self.file
            .lock()
            .seek(SeekFrom::Start((bid * BLK_LEN) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            self.file.lock().write(buf).unwrap(),
            BLK_LEN,
            "Not a complete block!"
        );
    }
}

impl FileDev {
    pub fn new(file: File) -> Self {
        Self {
            file: Spin::new(file),
        }
    }
}
