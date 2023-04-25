use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use fs::{dir::Dir, file::File};

use core::mem::size_of;
use fosix::signal::{SignalAction, SignalFlags};
use spin::{Spin, SpinGuard};

use crate::{
    config::NUM_SIGNAL,
    drivers::blockdev::BlkDev,
    mm::{address::VirAddr, memory::MemSet, page_table::PageTable},
    proc::{
        id::{GID_ALLOCATOR, PID_ALLOCATOR},
        manager::{INITPROC, PROC_MANAGER},
    },
    sync::{observable::Observable, semaphore::Semaphore},
    task::task::Task,
};

use super::{
    alloc_table::AllocTable,
    fd_table::FdTable,
    id::{Id, IdAllocator},
    lock_table::LockTable,
};

pub struct Proc {
    pid: Arc<Id>, // read only
    inner: Spin<ProcInner>,
}

pub struct ProcInner {
    tid_allocator: Arc<IdAllocator>,
    user_mem: MemSet,
    page_table: Arc<PageTable>,
    proc_staus: ProcState,
    parent: Option<Weak<Proc>>,
    children: Vec<Arc<Proc>>,
    tasks: Vec<Arc<Task>>,
    fd_table: FdTable,
    exit_code: isize,
    cwd: Dir<BlkDev>,
    sig_actions: [SignalAction; NUM_SIGNAL],
    base: VirAddr,
    lock_table: LockTable,
    sema_table: AllocTable<Arc<Semaphore>>,
    condvar_table: AllocTable<Arc<Observable>>,
    niceness: isize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcState {
    Running,
    Zombie,
}

impl Proc {
    /// Create a new task from elf data.
    pub fn from_elf(file: File<BlkDev>, parent: Option<Weak<Proc>>, niceness: isize) -> Arc<Self> {
        let file_size = file.lock().size();
        let mut elf_data = vec![0u8; file_size];
        assert_eq!(file.lock().read_at(&mut elf_data, 0), file_size);

        let page_table = Arc::new(PageTable::new());
        let (base, user_sepc, user_mem) = page_table.new_user(&elf_data);
        let tid_allocator = Arc::new(IdAllocator::new());

        println!("[trap] User's sepc is {:#x}", usize::from(user_sepc));

        let res = Arc::new(Self {
            pid: PID_ALLOCATOR.alloc(),
            inner: Spin::new(ProcInner {
                tid_allocator: tid_allocator.clone(),
                user_mem,
                page_table: page_table.clone(),
                proc_staus: ProcState::Running,
                parent,
                children: Vec::new(),
                tasks: Vec::new(),
                exit_code: 0,
                fd_table: FdTable::new(),
                cwd: file.lock().parent(),
                sig_actions: [SignalAction::default(); NUM_SIGNAL],
                base,
                lock_table: LockTable::new(),
                sema_table: AllocTable::new(),
                condvar_table: AllocTable::new(),
                niceness,
            }),
        });

        let task = Task::new(
            tid_allocator.alloc(),
            GID_ALLOCATOR.alloc(),
            base,
            user_sepc,
            Arc::downgrade(&res),
            page_table,
            res.lock().weight(),
        );
        res.lock().tasks.push(task);

        res
    }

    pub fn phantom(self: &Arc<Self>) -> Weak<Self> {
        Arc::downgrade(self)
    }

    pub fn lock(&self) -> SpinGuard<ProcInner> {
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
        task.exec(tid_allocator.alloc(), base, user_sepc, page_table.clone());
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

    /// Create a new thread that starts from `entry` with `arg` as its argument.
    pub fn new_task(self: &Arc<Self>, entry: VirAddr, arg: usize) -> Arc<Task> {
        let mut proc = self.lock();
        let task = Task::new(
            proc.tid_allocator.alloc(),
            GID_ALLOCATOR.alloc(),
            proc.base,
            entry,
            Arc::downgrade(self),
            proc.page_table.clone(),
            proc.weight(),
        );
        *task.lock().trap_ctx_mut().a0_mut() = arg;
        proc.tasks.push(task.clone());
        task
    }

    pub fn exit(&self, exit_code: isize) {
        let mut proc = self.lock();
        let pid = self.pid();

        proc.proc_staus = ProcState::Zombie;
        proc.exit_code = exit_code;
        proc.tasks = vec![];

        PROC_MANAGER.remove(pid);

        for child in proc.children().iter() {
            *child.lock().parent_mut() = Some(INITPROC.phantom());
            INITPROC.lock().children_mut().push(child.clone());
        }
        let parent = proc.parent().unwrap();
        parent.kill(SignalFlags::SIGCHLD);
        println!("[kernel] Process {} has ended.", pid);
    }

    pub fn kill(&self, sig: SignalFlags) {
        let proc = self.lock();
        let task = &proc.tasks[0];
        task.kill(sig);

        println!(
            "[kernel] Process {} receives signal {}",
            self.pid(),
            sig.bits()
        );
    }
}

impl Proc {
    /// Renew a process with a new page table. It's cloning that wrap inside `Arc`.
    fn renew(self: &Arc<Self>) -> Arc<Self> {
        let proc = self.lock();

        let page_table = Arc::new(PageTable::new());
        let user_mem = proc.user_mem.renew(&page_table);
        let base = proc.base;

        page_table.map_trampoline();

        let cwd = proc.cwd.clone();
        let fd_table = proc.fd_table().clone();
        let tid_allocator = Arc::new(IdAllocator::new());
        let parent = proc.parent.clone();
        let niceness = proc.niceness;
        assert_eq!(proc.lock_table.len(), 0); // the cloning of Spines is not supported yet
        assert_eq!(proc.sema_table.len(), 0); // the cloning of Spines is not supported yet

        let forked = Arc::new(Self {
            pid: PID_ALLOCATOR.alloc(),
            inner: Spin::new(ProcInner {
                tid_allocator: tid_allocator.clone(),
                user_mem,
                page_table: page_table.clone(),
                proc_staus: ProcState::Running,
                parent,
                children: Vec::new(),
                tasks: Vec::new(),
                exit_code: 0,
                fd_table,
                cwd,
                sig_actions: [SignalAction::default(); NUM_SIGNAL],
                base,
                lock_table: LockTable::new(),
                sema_table: AllocTable::new(),
                condvar_table: AllocTable::new(),
                niceness,
            }),
        });

        let tasks = proc
            .tasks
            .iter()
            .map(|task| {
                task.renew(
                    tid_allocator.alloc(),
                    GID_ALLOCATOR.alloc(),
                    Arc::downgrade(&forked),
                    page_table.clone(),
                    forked.lock().weight(),
                )
            })
            .collect();
        forked.inner.lock().tasks = tasks;
        forked
    }
}

impl ProcInner {
    pub fn proc_status_mut(&mut self) -> &mut ProcState {
        &mut self.proc_staus
    }

    pub fn proc_status(&self) -> ProcState {
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

    pub fn tasks_mut(&mut self) -> &mut Vec<Arc<Task>> {
        &mut self.tasks
    }

    pub fn lock_table(&self) -> &LockTable {
        &self.lock_table
    }

    pub fn lock_table_mut(&mut self) -> &mut LockTable {
        &mut self.lock_table
    }

    pub fn sema_table(&self) -> &AllocTable<Arc<Semaphore>> {
        &self.sema_table
    }

    pub fn sema_table_mut(&mut self) -> &mut AllocTable<Arc<Semaphore>> {
        &mut self.sema_table
    }

    pub fn condvar_table(&self) -> &AllocTable<Arc<Observable>> {
        &self.condvar_table
    }

    pub fn condvar_table_mut(&mut self) -> &mut AllocTable<Arc<Observable>> {
        &mut self.condvar_table
    }

    pub fn weight(&self) -> usize {
        if self.niceness < 0 {
            let positive = (-self.niceness) as u32;
            1024 * 5_usize.pow(positive) / 4_usize.pow(positive)
        } else {
            1024 * 4_usize.pow(self.niceness as u32) / 5_usize.pow(self.niceness as u32)
        }
    }
}
