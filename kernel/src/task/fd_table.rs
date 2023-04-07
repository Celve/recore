use core::ops::{Index, IndexMut};

use alloc::vec::Vec;

use crate::{
    fs::fileable::Fileable,
    io::{stdin::Stdin, stdout::Stdout},
};

#[derive(Clone)]
pub struct FdTable {
    entries: Vec<Option<Fileable>>,
}

impl FdTable {
    pub fn new() -> Self {
        Self {
            entries: vec![
                Some(Fileable::Stdin(Stdin)),
                Some(Fileable::Stdout(Stdout)),
                Some(Fileable::Stdout(Stdout)),
            ],
        }
    }

    pub fn alloc(&mut self, fileable: Fileable) -> usize {
        let pos = self.entries.iter().position(|x| x.is_none());
        if let Some(pos) = pos {
            self.entries[pos] = Some(fileable);
            pos
        } else {
            self.entries.push(Some(fileable));
            self.entries.len() - 1
        }
    }

    pub fn get(&self, fd: usize) -> Option<Fileable> {
        self.entries[fd].clone()
    }

    pub fn get_mut(&mut self, fd: usize) -> &mut Option<Fileable> {
        &mut self.entries[fd]
    }

    pub fn dealloc(&mut self, fd: usize) {
        self.entries[fd] = None;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
