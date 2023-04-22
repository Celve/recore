use crate::fs::FUSE;

use super::proc::Proc;

use fs::dir::Dir;

use alloc::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};
use fosix::fs::OpenFlags;
use lazy_static::lazy_static;
use spin::Spin;

pub struct ProcManager {
    /// The first task in the task deque is the next task, while the last task in the task deque is the current task.
    procs: Spin<BTreeMap<usize, Weak<Proc>>>,
}

impl ProcManager {
    pub fn new(proc: &Arc<Proc>) -> Self {
        let mut procs = BTreeMap::new();
        let pid = proc.pid();
        procs.insert(pid, proc.phantom());
        Self {
            procs: Spin::new(procs),
        }
    }

    pub fn push(&self, proc: &Arc<Proc>) {
        let pid = proc.pid();
        self.procs.lock().insert(pid, proc.phantom());
    }

    pub fn pop(&self) -> Option<Arc<Proc>> {
        self.procs
            .lock()
            .pop_first()
            .and_then(|(_, proc)| proc.upgrade())
    }

    pub fn remove(&self, pid: usize) {
        self.procs.lock().remove(&pid);
    }

    pub fn get(&self, key: usize) -> Option<Arc<Proc>> {
        self.procs.lock().get(&key).and_then(|proc| proc.upgrade())
    }
}

lazy_static! {
    pub static ref INITPROC: Arc<Proc> = Proc::from_elf(
        FUSE.root().lock().open("initproc", OpenFlags::RDONLY).unwrap(),
        None
    );

    /// Manager only loads the initproc at the beginning.
    pub static ref PROC_MANAGER: ProcManager = ProcManager::new(&INITPROC);
}
