use fosix::fs::{FileStat, SeekFlag};
use fs::{dir::Dir, file::File};

use crate::{
    drivers::blockdev::BlkDev,
    io::{stdin::Stdin, stdout::Stdout},
    ipc::pipe::Pipe,
};

use super::segment::Segment;

#[derive(Clone)]
pub enum Fileable {
    Stdin(Stdin),
    Stdout(Stdout),
    File(File<BlkDev>),
    Dir(Dir<BlkDev>),
    Pipe(Pipe),
}

impl Fileable {
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        match self {
            Fileable::Stdin(stdin) => stdin.read(buf),
            Fileable::File(file) => file.lock().read(buf),
            Fileable::Pipe(pipe) => pipe.read(buf),
            _ => 0,
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
            Fileable::File(file) => file.lock().write(buf),
            Fileable::Pipe(pipe) => pipe.write(buf),
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

    pub fn seek(&mut self, new_offset: usize, flag: SeekFlag) {
        match self {
            Fileable::File(file) => file.lock().seek(new_offset, flag),
            _ => {}
        }
    }

    pub fn stat(&self) -> FileStat {
        match self {
            Fileable::File(file) => file.lock().stat(),
            Fileable::Dir(dir) => dir.lock().stat(),
            _ => FileStat::empty(),
        }
    }
}

impl Fileable {
    pub fn as_file(&self) -> Option<File<BlkDev>> {
        match self {
            Fileable::File(file) => Some(file.clone()),
            _ => None,
        }
    }

    pub fn as_dir(&self) -> Option<Dir<BlkDev>> {
        match self {
            Fileable::Dir(dir) => Some(dir.clone()),
            _ => None,
        }
    }

    pub fn as_stdin(&self) -> Option<Stdin> {
        match self {
            Fileable::Stdin(stdin) => Some(*stdin),
            _ => None,
        }
    }

    pub fn as_stdout(&self) -> Option<Stdout> {
        match self {
            Fileable::Stdout(stdout) => Some(*stdout),
            _ => None,
        }
    }
}
