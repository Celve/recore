use crate::mm::memory::MemorySet;

use super::pid::{alloc_pid, Pid};

pub struct Task {
    pid: Pid,
    memory_set: MemorySet,
}

impl Task {
    pub fn from_elf(elf_data: &[u8]) -> Self {
        Self {
            pid: alloc_pid(),
            memory_set: MemorySet::from_elf(elf_data),
        }
    }
}
