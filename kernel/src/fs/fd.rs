use fosix::fs::FileStat;

use crate::io::{stdin::Stdin, stdout::Stdout};

use super::{
    dir::Dir,
    file::{File, Fileable},
};

pub enum FileDescriptor {
    Stdin(Stdin),
    Stdout(Stdout),
    File(File),
    Dir(Dir),
}

impl FileDescriptor {
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        match self {
            FileDescriptor::Stdin(stdin) => stdin.read(buf),
            FileDescriptor::Stdout(_) => 0,
            FileDescriptor::File(file) => file.read(buf),
            FileDescriptor::Dir(_) => 0,
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        match self {
            FileDescriptor::Stdin(_) => 0,
            FileDescriptor::Stdout(stdout) => stdout.write(buf),
            FileDescriptor::File(file) => file.write(buf),
            FileDescriptor::Dir(_) => 0,
        }
    }

    pub fn seek(&mut self, new_offset: usize) {
        match self {
            FileDescriptor::Stdin(_) => {}
            FileDescriptor::Stdout(_) => {}
            FileDescriptor::File(file) => file.seek(new_offset),
            FileDescriptor::Dir(_) => {}
        }
    }

    pub fn stat(&self) -> FileStat {
        match self {
            FileDescriptor::Stdin(_) => FileStat::empty(),
            FileDescriptor::Stdout(_) => FileStat::empty(),
            FileDescriptor::File(file) => file.stat(),
            FileDescriptor::Dir(dir) => dir.stat(),
        }
    }
}

impl FileDescriptor {
    pub fn as_file(&self) -> Option<&File> {
        match self {
            FileDescriptor::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn as_file_mut(&mut self) -> Option<&mut File> {
        match self {
            FileDescriptor::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn as_dir(&self) -> Option<&Dir> {
        match self {
            FileDescriptor::Dir(dir) => Some(dir),
            _ => None,
        }
    }

    pub fn as_dir_mut(&mut self) -> Option<&mut Dir> {
        match self {
            FileDescriptor::Dir(dir) => Some(dir),
            _ => None,
        }
    }
}
