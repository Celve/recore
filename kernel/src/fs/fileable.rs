use fosix::fs::FileStat;

use crate::io::{stdin::Stdin, stdout::Stdout};

use super::{dir::Dir, file::File, segment::Segment};

#[derive(Clone, Copy)]
pub enum Fileable {
    Stdin(Stdin),
    Stdout(Stdout),
    File(File),
    Dir(Dir),
}

impl Fileable {
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        match self {
            Fileable::Stdin(stdin) => stdin.read(buf),
            Fileable::Stdout(_) => 0,
            Fileable::File(file) => file.read(buf),
            Fileable::Dir(_) => 0,
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
            Fileable::Stdout(stdout) => stdout.write(buf),
            Fileable::File(file) => file.write(buf),
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
            Fileable::File(file) => file.seek(new_offset),
            _ => {}
        }
    }

    pub fn stat(&self) -> FileStat {
        match self {
            Fileable::File(file) => file.stat(),
            Fileable::Dir(dir) => dir.stat(),
            _ => FileStat::empty(),
        }
    }
}

impl Fileable {
    pub fn as_file(&self) -> Option<&File> {
        match self {
            Fileable::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn as_file_mut(&mut self) -> Option<&mut File> {
        match self {
            Fileable::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn as_dir(&self) -> Option<&Dir> {
        match self {
            Fileable::Dir(dir) => Some(dir),
            _ => None,
        }
    }

    pub fn as_dir_mut(&mut self) -> Option<&mut Dir> {
        match self {
            Fileable::Dir(dir) => Some(dir),
            _ => None,
        }
    }

    pub fn as_stdin(&self) -> Option<&Stdin> {
        match self {
            Fileable::Stdin(stdin) => Some(stdin),
            _ => None,
        }
    }

    pub fn as_stdin_mut(&mut self) -> Option<&mut Stdin> {
        match self {
            Fileable::Stdin(stdin) => Some(stdin),
            _ => None,
        }
    }

    pub fn as_stdout(&self) -> Option<&Stdout> {
        match self {
            Fileable::Stdout(stdout) => Some(stdout),
            _ => None,
        }
    }

    pub fn as_stdout_mut(&mut self) -> Option<&mut Stdout> {
        match self {
            Fileable::Stdout(stdout) => Some(stdout),
            _ => None,
        }
    }
}
