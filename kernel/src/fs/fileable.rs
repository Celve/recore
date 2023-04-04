use fosix::fs::FileStat;

use crate::io::{stdin::Stdin, stdout::Stdout};

use super::{dir::Dir, file::File, segment::Segment};

pub enum FileableX {
    Stdin(Stdin),
    Stdout(Stdout),
    File(File),
    Dir(Dir),
}

impl FileableX {
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        match self {
            FileableX::Stdin(stdin) => stdin.read(buf),
            FileableX::Stdout(_) => 0,
            FileableX::File(file) => file.read(buf),
            FileableX::Dir(_) => 0,
        }
    }

    pub fn read_seg(&mut self, seg: &mut Segment) -> usize {
        let mut bytes = 0;
        for buf in seg.iter_mut() {
            bytes += self.read(buf);
        }
        bytes
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        match self {
            FileableX::Stdout(stdout) => stdout.write(buf),
            FileableX::File(file) => file.write(buf),
            _ => 0,
        }
    }

    pub fn write_seg(&mut self, seg: &Segment) -> usize {
        let mut bytes = 0;
        for buf in seg.iter() {
            bytes += self.write(buf);
        }
        bytes
    }

    pub fn seek(&mut self, new_offset: usize) {
        match self {
            FileableX::File(file) => file.seek(new_offset),
            _ => {}
        }
    }

    pub fn stat(&self) -> FileStat {
        match self {
            FileableX::File(file) => file.stat(),
            FileableX::Dir(dir) => dir.stat(),
            _ => FileStat::empty(),
        }
    }
}

impl FileableX {
    pub fn as_file(&self) -> Option<&File> {
        match self {
            FileableX::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn as_file_mut(&mut self) -> Option<&mut File> {
        match self {
            FileableX::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn as_dir(&self) -> Option<&Dir> {
        match self {
            FileableX::Dir(dir) => Some(dir),
            _ => None,
        }
    }

    pub fn as_dir_mut(&mut self) -> Option<&mut Dir> {
        match self {
            FileableX::Dir(dir) => Some(dir),
            _ => None,
        }
    }
}
