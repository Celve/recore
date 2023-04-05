use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use riscv::register::sstatus::{self, SPP};
use spin::mutex::{Mutex, MutexGuard};

use crate::{
    config::TRAP_CONTEXT_START_ADDRESS,
    fs::{dir::Dir, file::File, fileable::Fileable},
    io::{stdin::Stdin, stdout::Stdout},
    mm::{
        address::{PhyAddr, VirAddr},
        memory::{Memory, KERNEL_SPACE},
    },
    trap::{context::TrapContext, trampoline::restore, trap_handler},
};

use super::{
    fd_table::FdTable,
    pid::{alloc_pid, Pid},
};
use crate::task::stack::KernelStack;

pub struct Task {
    inner: Mutex<TaskInner>,
}

pub struct TaskInner {
    pid: Pid,
    user_mem: Memory,
    task_status: TaskStatus,
    task_ctx: TaskContext,
    trap_ctx: PhyAddr, // raw pointer can't be shared between threads, therefore use phyaddr instead
    kernel_stack: KernelStack,
    parent: Option<Weak<Task>>,
    children: Vec<Arc<Task>>,
    fd_table: FdTable,
    exit_code: isize,
    cwd: Dir,
}

#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub sr: [usize; 12],
}

#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}

impl Task {
    /// Create a new task from elf data.
    pub fn from_elf(file: File, parent: Option<Weak<Task>>) -> Self {
        let file_size = file.size();
        let mut elf_data = vec![0u8; file_size];
        assert_eq!(file.read_at(&mut elf_data, 0), file_size);

        let pid = alloc_pid();
        let (user_mem, user_sp, user_sepc) = Memory::from_elf(&elf_data);
        let page_table = user_mem.page_table();

        let trap_ctx = page_table
            .translate_vpn(VirAddr::from(TRAP_CONTEXT_START_ADDRESS).floor_to_vir_page_num())
            .expect("[task] Unable to access trap context.")
            .get_ppn();
        println!(
            "[trap] Map {:#x} -> {:#x}",
            usize::from(VirAddr::from(TRAP_CONTEXT_START_ADDRESS)),
            usize::from(trap_ctx)
        );
        println!("[trap] User's sepc is {:#x}", user_sepc);

        let kernel_stack = KernelStack::new(pid.0);

        let raw_trap_ctx = trap_ctx.as_raw_bytes() as *mut [u8] as *mut TrapContext;
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        unsafe {
            *raw_trap_ctx = TrapContext::new(
                user_sp,
                user_sepc,
                sstatus.bits(),
                kernel_stack.top().into(),
                trap_handler as usize,
                KERNEL_SPACE.lock().page_table().to_satp(),
            );
        }
        Self {
            inner: Mutex::new(TaskInner {
                pid,
                user_mem,
                task_status: TaskStatus::Ready,
                task_ctx: TaskContext::new(restore as usize, kernel_stack.top().into()),
                trap_ctx: trap_ctx.into(),
                kernel_stack,
                parent,
                children: Vec::new(),
                exit_code: 0,
                fd_table: FdTable::new(),
                cwd: file.parent(),
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
    pub fn exec(&self, file: File) {
        let file_size = file.size();
        let mut elf_data = vec![0u8; file_size];
        assert_eq!(file.read_at(&mut elf_data, 0), file_size);

        let mut task = self.lock();
        let (user_mem, user_sp, user_sepc) = Memory::from_elf(&elf_data);
        let page_table = user_mem.page_table();
        let trap_ctx = page_table
            .translate_vpn(VirAddr::from(TRAP_CONTEXT_START_ADDRESS).floor_to_vir_page_num())
            .expect("[task] Unable to access trap context.")
            .get_ppn();
        let raw_trap_ctx = trap_ctx.as_raw_bytes() as *mut [u8] as *mut TrapContext;
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        unsafe {
            *raw_trap_ctx = TrapContext::new(
                user_sp,
                user_sepc,
                sstatus.bits(),
                task.kernel_stack.top().into(),
                trap_handler as usize,
                KERNEL_SPACE.lock().page_table().to_satp(),
            );
        }

        // replace some
        task.user_mem = user_mem;
        task.trap_ctx = trap_ctx.into();
        task.task_ctx = TaskContext::new(restore as usize, task.kernel_stack.top().into());
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        let task = self.lock();
        let pid = alloc_pid();
        let user_mem = task.user_mem().clone();
        let page_table = user_mem.page_table();
        let trap_ctx = page_table
            .translate_vpn(VirAddr::from(TRAP_CONTEXT_START_ADDRESS).floor_to_vir_page_num())
            .expect("[task] Unable to access trap context.")
            .get_ppn();
        let kernel_stack = KernelStack::new(pid.0);
        let cwd = task.cwd;
        let fd_table = task.fd_table().clone();

        // we have to modify the kernel sp both in trap ctx and task ctx
        let raw_trap_ctx = trap_ctx.as_raw_bytes() as *mut [u8] as *mut TrapContext;
        unsafe {
            (*raw_trap_ctx).kernel_sp = kernel_stack.top().into();
        }

        Self {
            inner: Mutex::new(TaskInner {
                pid,
                user_mem,
                task_status: TaskStatus::Ready,
                task_ctx: TaskContext::new(restore as usize, kernel_stack.top().into()),
                trap_ctx: trap_ctx.into(),
                kernel_stack,
                parent: task.parent.clone(),
                children: Vec::new(),
                exit_code: 0,
                fd_table,
                cwd,
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

    pub fn trap_ctx_ptr(&self) -> *mut TrapContext {
        self.trap_ctx.as_mut().unwrap()
    }

    pub fn trap_ctx_ref(&self) -> *const TrapContext {
        self.trap_ctx.as_ref().unwrap()
    }

    pub fn trap_ctx(&self) -> &TrapContext {
        self.trap_ctx.as_ref().unwrap()
    }

    pub fn trap_ctx_mut(&self) -> &mut TrapContext {
        self.trap_ctx.as_mut().unwrap()
    }

    pub fn task_status_mut(&mut self) -> &mut TaskStatus {
        &mut self.task_status
    }

    pub fn task_status(&self) -> &TaskStatus {
        &self.task_status
    }

    pub fn exit_code_mut(&mut self) -> &mut isize {
        &mut self.exit_code
    }

    pub fn user_mem(&self) -> &Memory {
        &self.user_mem
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

    pub fn parent_mut(&mut self) -> &mut Option<Weak<Task>> {
        &mut self.parent
    }

    pub fn exit_code(&self) -> isize {
        self.exit_code
    }

    pub fn cwd(&self) -> Dir {
        self.cwd
    }

    pub fn cwd_mut(&mut self) -> &mut Dir {
        &mut self.cwd
    }

    pub fn fd_table(&self) -> &FdTable {
        &self.fd_table
    }

    pub fn fd_table_mut(&mut self) -> &mut FdTable {
        &mut self.fd_table
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
