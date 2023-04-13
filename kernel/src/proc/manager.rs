use super::{id::Id, proc::Proc};

use crate::fs::FUSE;

use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};
use fosix::fs::OpenFlags;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

pub struct ProcManager {
    /// The first task in the task deque is the next task, while the last task in the task deque is the current task.
    procs: Mutex<BTreeMap<usize, Arc<Proc>>>,
}

impl ProcManager {
    pub fn new(proc: Arc<Proc>) -> Self {
        let mut procs = BTreeMap::new();
        let pid = proc.pid();
        procs.insert(pid, proc);
        Self {
            procs: Mutex::new(procs),
        }
    }

    pub fn push(&self, proc: Arc<Proc>) {
        let pid = proc.pid();
        self.procs.lock().insert(pid, proc);
    }

    pub fn pop(&self) -> Option<Arc<Proc>> {
        self.procs.lock().pop_first().map(|(_, proc)| proc)
    }

    pub fn remove(&self, pid: usize) {
        self.procs.lock().remove(&pid);
    }

    pub fn get(&self, key: usize) -> Option<Arc<Proc>> {
        self.procs.lock().get(&key).cloned()
    }
}

lazy_static! {
    pub static ref INITPROC: Arc<Proc> = Proc::from_elf(
        FUSE.root().lock().open("initproc", OpenFlags::RDONLY).unwrap(),
        None
    );

    /// Manager only loads the initproc at the beginning.
    pub static ref PROC_MANAGER: ProcManager = ProcManager::new(INITPROC.clone());
}
