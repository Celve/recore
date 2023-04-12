use core::mem::size_of;

use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use fosix::signal::{SignalAction, SignalFlags};
use fs::{dir::Dir, file::File};
use riscv::register::sstatus::{self, SPP};
use spin::mutex::{Mutex, MutexGuard};

use crate::{
    config::NUM_SIGNAL,
    fs::disk::BlkDev,
    mm::{
        address::{PhyAddr, VirAddr},
        memory::MemSet,
        page_table::{PageTable, KERNEL_PAGE_TABLE},
    },
    trap::{
        context::{TrapCtx, TrapCtxHandle},
        trampoline::restore,
        trap_handler,
    },
};

use super::{
    fd_table::FdTable,
    pid::{alloc_pid, Pid},
    stack::UserStack,
};
use crate::task::stack::KernelStack;

pub struct Task {
    inner: Mutex<TaskInner>,
}

pub struct TaskInner {
    pid: Pid,
    user_mem: MemSet,
    page_table: Arc<PageTable>,
    task_status: TaskStatus,
    task_ctx: TaskContext,
    trap_ctx_handle: TrapCtxHandle,
    trap_ctx_backup: Option<TrapCtx>,
    user_stack: UserStack,
    kernel_stack: KernelStack,
    parent: Option<Weak<Task>>,
    children: Vec<Arc<Task>>,
    fd_table: FdTable,
    exit_code: isize,
    cwd: Dir<BlkDev>,
    sig: SignalFlags,
    sig_mask: SignalFlags,
    sig_actions: [SignalAction; NUM_SIGNAL],
    sig_handling: Option<usize>,
    base: VirAddr,
}

#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub sr: [usize; 12],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Stopped,
    Zombie,
}

impl Task {
    /// Create a new task from elf data.
    pub fn from_elf(file: File<BlkDev>, parent: Option<Weak<Task>>) -> Self {
        let file_size = file.lock().size();
        let mut elf_data = vec![0u8; file_size];
        assert_eq!(file.lock().read_at(&mut elf_data, 0), file_size);

        let page_table = Arc::new(PageTable::new());
        let (base, user_sepc, user_mem, mut trap_ctx_handle, user_stack) =
            page_table.new_user(&elf_data);

        println!("[trap] User's sepc is {:#x}", usize::from(user_sepc));

        let pid = alloc_pid();
        let kernel_stack = KERNEL_PAGE_TABLE.new_kernel_stack(pid.0);

        let trap_ctx = trap_ctx_handle.trap_ctx_mut();
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        *trap_ctx = TrapCtx::new(
            user_stack.top().into(),
            user_sepc.into(),
            sstatus.bits(),
            kernel_stack.top().into(),
            trap_handler as usize,
            KERNEL_PAGE_TABLE.to_satp(),
        );

        Self {
            inner: Mutex::new(TaskInner {
                pid,
                user_mem,
                page_table,
                task_status: TaskStatus::Ready,
                task_ctx: TaskContext::new(restore as usize, kernel_stack.top().into()),
                trap_ctx_handle,
                trap_ctx_backup: None,
                user_stack,
                kernel_stack,
                parent,
                children: Vec::new(),
                exit_code: 0,
                fd_table: FdTable::new(),
                cwd: file.lock().parent(),
                sig: SignalFlags::from_bits(0).unwrap(),
                sig_mask: SignalFlags::from_bits(0).unwrap(),
                sig_actions: [SignalAction::default(); NUM_SIGNAL],
                sig_handling: None,
                base,
            }),
        }
    }

    pub fn lock(&self) -> MutexGuard<TaskInner> {
        self.inner.lock()
    }

    /// Fork a new task from an existing task with both of the return values unchanged.  
    ///
    /// The difference between this function and clone is that it maintains the parent-child relationship.
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let new_task = Arc::new(self.as_ref().clone());

        // make new process the original's children
        self.lock().children_mut().push(new_task.clone());
        *new_task.lock().parent_mut() = Some(Arc::downgrade(&self));

        new_task
    }

    /// Replace the current task with new elf data. Therefore, all user configurations would be reset.
    pub fn exec(&self, file: File<BlkDev>, args: &Vec<String>) {
        let file_size = file.lock().size();
        let mut elf_data = vec![0u8; file_size];
        assert_eq!(file.lock().read_at(&mut elf_data, 0), file_size);

        let mut task = self.lock();
        let page_table = Arc::new(PageTable::new());
        let (base, user_sepc, user_mem, mut trap_ctx_handle, user_stack) =
            page_table.new_user(&elf_data);
        let mut user_sp: usize = user_stack.top().into();

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

        let trap_ctx = trap_ctx_handle.trap_ctx_mut();
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        *trap_ctx = TrapCtx::new(
            user_sp,
            user_sepc.into(),
            sstatus.bits(),
            task.kernel_stack.top().into(),
            trap_handler as usize,
            KERNEL_PAGE_TABLE.to_satp(),
        );

        // replace some
        task.page_table = page_table;
        task.base = base;
        task.user_mem = user_mem;
        task.user_stack = user_stack;
        task.trap_ctx_handle = trap_ctx_handle;
        *task.trap_ctx_mut().a0_mut() = args.len();
        *task.trap_ctx_mut().a1_mut() = argv;
        task.task_ctx = TaskContext::new(restore as usize, task.kernel_stack.top().into());
    }

    pub fn exit(&self, exit_code: isize) {
        let mut task = self.lock();
        task.task_status = TaskStatus::Zombie;
        task.exit_code = exit_code;
    }

    pub fn stop(&self) {
        let mut task = self.lock();
        task.task_status = TaskStatus::Stopped;
    }

    pub fn cont(&self) {
        let mut task = self.lock();
        task.task_status = TaskStatus::Running;
    }

    pub fn kill(&self, sig: SignalFlags) {
        let mut task = self.lock();
        task.sig |= sig;
        println!(
            "[kernel] Process {} receives signal {}",
            task.pid(),
            sig.bits()
        );
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        let task = self.lock();

        let page_table = Arc::new(PageTable::new());
        let user_mem = task.user_mem.renew(&page_table);
        let mut trap_ctx_handle = task.trap_ctx_handle.renew(&page_table);
        let user_stack = task.user_stack.renew(&page_table);
        page_table.map_trampoline();

        let pid = alloc_pid();
        let kernel_stack = KERNEL_PAGE_TABLE.new_kernel_stack(pid.0);

        let cwd = task.cwd.clone();
        let fd_table = task.fd_table().clone();

        // we have to modify the kernel sp both in trap ctx and task ctx
        let trap_ctx = trap_ctx_handle.trap_ctx_mut();
        trap_ctx.kernel_sp = kernel_stack.top().into();

        Self {
            inner: Mutex::new(TaskInner {
                pid,
                user_mem,
                page_table,
                task_status: TaskStatus::Ready,
                task_ctx: TaskContext::new(restore as usize, kernel_stack.top().into()),
                trap_ctx_handle,
                trap_ctx_backup: None,
                kernel_stack,
                user_stack,
                parent: task.parent.clone(),
                children: Vec::new(),
                exit_code: 0,
                fd_table,
                cwd,
                sig: SignalFlags::from_bits(0).unwrap(),
                sig_mask: SignalFlags::from_bits(0).unwrap(),
                sig_actions: [SignalAction::default(); NUM_SIGNAL],
                sig_handling: None,
                base: task.base,
            }),
        }
    }
}

impl TaskInner {
    pub fn task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.task_ctx
    }

    pub fn task_ctx_mut(&mut self) -> &mut TaskContext {
        &mut self.task_ctx
    }

    pub fn task_ctx(&self) -> &TaskContext {
        &self.task_ctx
    }

    pub fn trap_ctx(&self) -> &TrapCtx {
        self.trap_ctx_handle.trap_ctx()
    }

    pub fn trap_ctx_handle(&self) -> &TrapCtxHandle {
        &self.trap_ctx_handle
    }

    pub fn trap_ctx_handle_mut(&mut self) -> &mut TrapCtxHandle {
        &mut self.trap_ctx_handle
    }

    pub fn trap_ctx_mut(&mut self) -> &mut TrapCtx {
        self.trap_ctx_handle.trap_ctx_mut()
    }

    pub fn task_status_mut(&mut self) -> &mut TaskStatus {
        &mut self.task_status
    }

    pub fn task_status(&self) -> TaskStatus {
        self.task_status
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

    pub fn pid(&self) -> usize {
        self.pid.0
    }

    pub fn children_mut(&mut self) -> &mut Vec<Arc<Task>> {
        &mut self.children
    }

    pub fn children(&self) -> &Vec<Arc<Task>> {
        &self.children
    }

    pub fn parent(&self) -> Option<Arc<Task>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn parent_mut(&mut self) -> &mut Option<Weak<Task>> {
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

    pub fn sig(&self) -> SignalFlags {
        self.sig
    }

    pub fn sig_mut(&mut self) -> &mut SignalFlags {
        &mut self.sig
    }

    pub fn sig_mask(&self) -> SignalFlags {
        self.sig_mask
    }

    pub fn sig_mask_mut(&mut self) -> &mut SignalFlags {
        &mut self.sig_mask
    }

    pub fn sig_actions(&self) -> &[SignalAction; NUM_SIGNAL] {
        &self.sig_actions
    }

    pub fn sig_actions_mut(&mut self) -> &mut [SignalAction; NUM_SIGNAL] {
        &mut self.sig_actions
    }

    pub fn sig_handling(&self) -> Option<usize> {
        self.sig_handling
    }

    pub fn sig_handling_mut(&mut self) -> &mut Option<usize> {
        &mut self.sig_handling
    }

    pub fn trap_ctx_backup(&self) -> &Option<TrapCtx> {
        &self.trap_ctx_backup
    }

    pub fn trap_ctx_backup_mut(&mut self) -> &mut Option<TrapCtx> {
        &mut self.trap_ctx_backup
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
