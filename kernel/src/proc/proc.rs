use core::mem::size_of;

use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use fosix::signal::{SignalAction, SignalFlags};
use fs::{dir::Dir, file::File};
use spin::mutex::{Mutex, MutexGuard};

use crate::{
    config::NUM_SIGNAL,
    fs::disk::BlkDev,
    mm::{address::VirAddr, memory::MemSet, page_table::PageTable},
    proc::id::{GID_ALLOCATOR, PID_ALLOCATOR},
    task::task::Task,
};

use super::{
    fd_table::FdTable,
    id::{Id, IdAllocator},
};

pub struct Proc {
    pid: Arc<Id>, // read only
    inner: Mutex<ProcInner>,
}

pub struct ProcInner {
    tid_allocator: Arc<IdAllocator>,
    user_mem: MemSet,
    page_table: Arc<PageTable>,
    proc_staus: ProcStatus,
    parent: Option<Weak<Proc>>,
    children: Vec<Arc<Proc>>,
    tasks: Vec<Arc<Task>>,
    fd_table: FdTable,
    exit_code: isize,
    cwd: Dir<BlkDev>,
    sig_actions: [SignalAction; NUM_SIGNAL],
    base: VirAddr,
}

#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub sr: [usize; 12],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcStatus {
    Ready,
    Running,
    Stopped,
    Zombie,
}

impl Proc {
    /// Create a new task from elf data.
    pub fn from_elf(file: File<BlkDev>, parent: Option<Weak<Proc>>) -> Arc<Self> {
        let file_size = file.lock().size();
        let mut elf_data = vec![0u8; file_size];
        assert_eq!(file.lock().read_at(&mut elf_data, 0), file_size);

        let page_table = Arc::new(PageTable::new());
        let (base, user_sepc, user_mem) = page_table.new_user(&elf_data);
        let tid_allocator = Arc::new(IdAllocator::new());

        println!("[trap] User's sepc is {:#x}", usize::from(user_sepc));

        let res = Arc::new(Self {
            pid: PID_ALLOCATOR.alloc(),
            inner: Mutex::new(ProcInner {
                tid_allocator: tid_allocator.clone(),
                user_mem,
                page_table: page_table.clone(),
                proc_staus: ProcStatus::Ready,
                parent,
                children: Vec::new(),
                tasks: Vec::new(),
                exit_code: 0,
                fd_table: FdTable::new(),
                cwd: file.lock().parent(),
                sig_actions: [SignalAction::default(); NUM_SIGNAL],
                base,
            }),
        });

        let task = Arc::new(Task::new(
            tid_allocator.alloc(),
            GID_ALLOCATOR.alloc(),
            base,
            user_sepc,
            Arc::downgrade(&res),
            page_table,
        ));
        res.lock().tasks.push(task);

        res
    }

    pub fn lock(&self) -> MutexGuard<ProcInner> {
        self.inner.lock()
    }

    pub fn pid(&self) -> usize {
        self.pid.id()
    }

    /// Fork a new task from an existing task with both of the return values unchanged.  
    ///
    /// The difference between this function and clone is that it maintains the parent-child relationship.
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let new_proc = self.renew();

        // make new process the original's children
        self.lock().children_mut().push(new_proc.clone());
        *new_proc.lock().parent_mut() = Some(Arc::downgrade(&self));

        new_proc
    }

    /// Replace the current task with new elf data. Therefore, all user configurations would be reset.
    pub fn exec(self: &Arc<Self>, file: File<BlkDev>, args: &Vec<String>) {
        let file_size = file.lock().size();
        let mut elf_data = vec![0u8; file_size];
        assert_eq!(file.lock().read_at(&mut elf_data, 0), file_size);

        let mut proc = self.lock();
        let page_table = Arc::new(PageTable::new());
        let (base, user_sepc, user_mem) = page_table.new_user(&elf_data);
        let tid_allocator = Arc::new(IdAllocator::new());
        let task = proc.main_task();
        task.exec(base, user_sepc, page_table.clone());
        let mut user_sp: usize = task.lock().user_stack().top().into();

        // push args
        let mut acc = 0;
        for i in (0..=args.len()).rev() {
            let ptr = if i == args.len() {
                0
            } else {
                user_sp - (args.len() + 1) * size_of::<usize>() - acc - args[i].len()
            };
            let offset = (args.len() + 1 - i) * size_of::<usize>();
            let src_bytes = ptr.to_ne_bytes();
            let mut dst_bytes =
                page_table.translate_bytes((user_sp - offset).into(), src_bytes.len());
            println!("write {:#x} in {:#x}", ptr, user_sp - offset);
            dst_bytes
                .iter_mut()
                .enumerate()
                .for_each(|(i, byte)| **byte = src_bytes[i]);

            if i != args.len() {
                acc += args[i].len();
            }
        }
        user_sp -= (args.len() + 1) * size_of::<usize>();
        let argv = user_sp;

        for arg in args.iter().rev() {
            let src_bytes = arg.as_bytes();
            user_sp -= src_bytes.len();
            let mut dst_bytes = page_table.translate_bytes(user_sp.into(), src_bytes.len());
            dst_bytes
                .iter_mut()
                .enumerate()
                .for_each(|(i, byte)| **byte = src_bytes[i]);
        }

        println!(
            "page_table: {:#x} result: {:#x}",
            page_table.to_satp(),
            page_table.translate_any::<usize>(0x29ff0.into())
        );

        // replace some
        *task.lock().trap_ctx_mut().a0_mut() = args.len();
        *task.lock().trap_ctx_mut().a1_mut() = argv;
        *task.lock().trap_ctx_mut().user_sp_mut() = user_sp.into();

        proc.page_table = page_table;
        proc.base = base;
        proc.user_mem = user_mem;
        proc.tid_allocator = tid_allocator;
        proc.tasks = vec![task];
    }

    pub fn exit(&self, exit_code: isize) {
        let mut proc = self.lock();
        proc.proc_staus = ProcStatus::Zombie;
        proc.exit_code = exit_code;
    }

    pub fn stop(&self) {
        let mut proc = self.lock();
        proc.proc_staus = ProcStatus::Stopped;
    }

    pub fn cont(&self) {
        let mut proc = self.lock();
        proc.proc_staus = ProcStatus::Running;
    }

    pub fn kill(&self, sig: SignalFlags) {
        let proc = self.lock();
        *proc.tasks[0].lock().sigs_mut() |= sig;
        println!(
            "[kernel] Process {} receives signal {}",
            self.pid(),
            sig.bits()
        );
    }
}

impl Proc {
    fn renew(self: &Arc<Self>) -> Arc<Self> {
        let proc = self.lock();

        let page_table = Arc::new(PageTable::new());
        let user_mem = proc.user_mem.renew(&page_table);
        let base = proc.base;

        page_table.map_trampoline();

        let cwd = proc.cwd.clone();
        let fd_table = proc.fd_table().clone();

        let forked = Arc::new(Self {
            pid: PID_ALLOCATOR.alloc(),
            inner: Mutex::new(ProcInner {
                tid_allocator: Arc::new(IdAllocator::new()),
                user_mem,
                page_table: page_table.clone(),
                proc_staus: ProcStatus::Ready,
                parent: proc.parent.clone(),
                children: Vec::new(),
                tasks: Vec::new(),
                exit_code: 0,
                fd_table,
                cwd,
                sig_actions: [SignalAction::default(); NUM_SIGNAL],
                base: proc.base,
            }),
        });

        let tasks = proc
            .tasks
            .iter()
            .map(|task| {
                Arc::new(task.renew(
                    GID_ALLOCATOR.alloc(),
                    base,
                    Arc::downgrade(&forked),
                    page_table.clone(),
                ))
            })
            .collect();
        forked.inner.lock().tasks = tasks;
        forked
    }
}

impl ProcInner {
    pub fn proc_status_mut(&mut self) -> &mut ProcStatus {
        &mut self.proc_staus
    }

    pub fn proc_status(&self) -> ProcStatus {
        self.proc_staus
    }

    pub fn exit_code_mut(&mut self) -> &mut isize {
        &mut self.exit_code
    }

    pub fn user_mem(&self) -> &MemSet {
        &self.user_mem
    }

    pub fn page_table(&self) -> Arc<PageTable> {
        self.page_table.clone()
    }

    pub fn children_mut(&mut self) -> &mut Vec<Arc<Proc>> {
        &mut self.children
    }

    pub fn children(&self) -> &Vec<Arc<Proc>> {
        &self.children
    }

    pub fn parent(&self) -> Option<Arc<Proc>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn parent_mut(&mut self) -> &mut Option<Weak<Proc>> {
        &mut self.parent
    }

    pub fn exit_code(&self) -> isize {
        self.exit_code
    }

    pub fn cwd(&self) -> Dir<BlkDev> {
        self.cwd.clone()
    }

    pub fn cwd_mut(&mut self) -> &mut Dir<BlkDev> {
        &mut self.cwd
    }

    pub fn fd_table(&self) -> &FdTable {
        &self.fd_table
    }

    pub fn fd_table_mut(&mut self) -> &mut FdTable {
        &mut self.fd_table
    }

    pub fn sig_actions(&self) -> &[SignalAction; NUM_SIGNAL] {
        &self.sig_actions
    }

    pub fn sig_actions_mut(&mut self) -> &mut [SignalAction; NUM_SIGNAL] {
        &mut self.sig_actions
    }

    pub fn main_task(&self) -> Arc<Task> {
        self.tasks[0].clone()
    }

    pub fn tasks(&self) -> &Vec<Arc<Task>> {
        &self.tasks
    }
}

impl TaskContext {
    pub fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            sr: [0; 12],
        }
    }

    pub fn new(ra: usize, sp: usize) -> Self {
        Self {
            ra,
            sp,
            sr: [0; 12],
        }
    }
}
