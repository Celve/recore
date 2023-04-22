use alloc::sync::{Arc, Weak};
use fosix::signal::SignalFlags;
use riscv::register::sstatus::{self, SPP};
use spin::{Spin, SpinGuard};

use crate::{
    mm::{
        address::VirAddr,
        page_table::{PageTable, KERNEL_PAGE_TABLE},
    },
    proc::{
        id::Id,
        proc::Proc,
        stack::{KernelStack, UserStack},
    },
    task::{manager::TASK_MANAGER, processor::fetch_idle_task_ctx_ptr},
    trap::{
        context::{TrapCtx, TrapCtxHandle},
        fail,
        trampoline::restore,
        trap_handler,
    },
};

use super::context::TaskContext;

pub struct Task {
    proc: Weak<Proc>,
    inner: Spin<TaskInner>,
}

pub struct TaskInner {
    tid: Arc<Id>,
    gid: Arc<Id>,
    page_table: Arc<PageTable>,
    task_state: TaskState,
    task_ctx: TaskContext,
    trap_ctx_handle: TrapCtxHandle,
    trap_ctx_backup: Option<TrapCtx>,
    user_stack: UserStack,
    kernel_stack: KernelStack,
    exit_code: isize,
    sigs: SignalFlags,
    sig_mask: SignalFlags,
    sig_handling: Option<usize>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TaskState {
    Ready,
    Running,
    Stopped,
    Zombie,
}

impl Task {
    pub fn new(
        tid: Arc<Id>,
        gid: Arc<Id>,
        base: VirAddr,
        entry: VirAddr,
        proc: Weak<Proc>,
        page_table: Arc<PageTable>,
    ) -> Self {
        let kernel_stack = KernelStack::new(gid.id());
        let task_ctx = TaskContext::new(restore as usize, kernel_stack.top().into());
        let trap_ctx_handle = page_table.new_trap_ctx(tid.id());
        let user_stack = page_table.new_user_stack(base, tid.id());

        let trap_ctx = trap_ctx_handle.trap_ctx_mut();
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        *trap_ctx = TrapCtx::new(
            user_stack.top().into(),
            entry.into(),
            sstatus.bits(),
            kernel_stack.top().into(),
            trap_handler as usize,
            KERNEL_PAGE_TABLE.to_satp(),
        );

        Self {
            inner: Spin::new(TaskInner {
                tid,
                gid,
                page_table,
                task_state: TaskState::Running,
                task_ctx,
                trap_ctx_handle,
                trap_ctx_backup: None,
                user_stack,
                kernel_stack,
                exit_code: 0,
                sigs: SignalFlags::empty(),
                sig_mask: SignalFlags::empty(),
                sig_handling: None,
            }),
            proc,
        }
    }

    pub fn renew(
        &self,
        tid: Arc<Id>,
        gid: Arc<Id>,
        proc: Weak<Proc>,
        page_table: Arc<PageTable>,
    ) -> Self {
        assert_eq!(self.lock().tid.id(), tid.id());
        let kernel_stack = KernelStack::new(gid.id());
        let task_ctx = TaskContext::new(restore as usize, kernel_stack.top().into());
        let trap_ctx_handle = self.lock().trap_ctx_handle.renew(&page_table);
        let trap_ctx_backup = self.lock().trap_ctx_backup.clone();
        let user_stack = self.lock().user_stack.renew(&page_table);

        let trap_ctx = trap_ctx_handle.trap_ctx_mut();
        trap_ctx.kernel_sp = kernel_stack.top().into();

        Self {
            inner: Spin::new(TaskInner {
                tid,
                gid,
                page_table,
                task_state: TaskState::Running,
                task_ctx,
                trap_ctx_handle,
                trap_ctx_backup,
                user_stack,
                kernel_stack,
                exit_code: 0,
                sigs: SignalFlags::empty(), // FIX: inherit signal
                sig_mask: SignalFlags::empty(),
                sig_handling: None,
            }),
            proc,
        }
    }

    pub fn exec(&self, tid: Arc<Id>, base: VirAddr, entry: VirAddr, page_table: Arc<PageTable>) {
        let mut task = self.lock();

        let task_ctx = TaskContext::new(restore as usize, task.kernel_stack.top().into());
        let trap_ctx_handle = page_table.new_trap_ctx(task.tid());
        let user_stack = page_table.new_user_stack(base, task.tid());

        let trap_ctx = trap_ctx_handle.trap_ctx_mut();
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        *trap_ctx = TrapCtx::new(
            user_stack.top().into(),
            entry.into(),
            sstatus.bits(),
            task.kernel_stack.top().into(),
            trap_handler as usize,
            KERNEL_PAGE_TABLE.to_satp(),
        );

        task.tid = tid;
        task.task_ctx = task_ctx;
        task.trap_ctx_handle = trap_ctx_handle;
        task.user_stack = user_stack;
        task.page_table = page_table;
    }

    pub fn lock(&self) -> SpinGuard<TaskInner> {
        self.inner.lock()
    }

    pub fn proc(&self) -> Arc<Proc> {
        self.proc.clone().upgrade().unwrap()
    }
}

impl Task {
    pub fn phantom(self: &Arc<Self>) -> Weak<Self> {
        Arc::downgrade(self)
    }
}

impl Task {
    /// Suspend the task.
    ///
    /// When `suspend()` is called, the task would never be put into the task manager again.
    /// There should be other structure that holds the task, and it should wake up the task when needed.
    pub fn suspend(&self) {
        self.lock().task_state = TaskState::Stopped;
        self.schedule();
    }

    /// Yield the task.
    ///
    /// It's not like the `suspend()', because it would be put into the task manager when called.
    pub fn yield_now(self: &Arc<Self>) {
        self.schedule();
    }

    /// Exit the task.
    ///
    /// It directly exits the task by setting the state and exit code.
    /// It's illegal to put this task to the task manager again.
    pub fn exit(&self, exit_code: isize) {
        println!(
            "process {} thread {} exit with code {}",
            self.proc().pid(),
            self.lock().tid(),
            exit_code
        );
        {
            let mut task = self.lock();
            task.task_state = TaskState::Zombie;
            task.exit_code = exit_code;

            // in case that it's the main thread
            if task.tid() == 1 {
                self.proc().exit(exit_code);
            }
        }
        self.schedule()
    }

    /// Wake up the task.
    ///
    /// It's the companion method with `suspend()`.
    /// When the `suspend()` is called, the caller is reponsible to maintain the task elsewhere.
    /// Then the caller should wake up the task by calling this function, which would put the task into task manager again.
    pub fn wake_up(self: &Arc<Self>) {
        self.lock().task_state = TaskState::Running;
        TASK_MANAGER.push(self);
    }

    /// Schedule the task, namely giving back control to the processor's idle thread.
    fn schedule(&self) {
        let task_ctx = self.lock().task_ctx_ptr();
        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        if (task_ctx as usize) < 10 {
            fail();
        }
        unsafe { _switch(task_ctx, fetch_idle_task_ctx_ptr()) }
    }
}

impl Task {
    /// Append the task's signal flags by locking it.
    pub fn kill(self: &Arc<Self>, sig: SignalFlags) {
        let mut task = self.lock();
        task.sigs |= sig;

        println!(
            "[kernel] Thread {} in process {} receives signal {}",
            task.tid(),
            self.proc.upgrade().unwrap().pid(),
            sig.bits()
        );

        if sig.contains(SignalFlags::SIGCONT) && task.task_state() == TaskState::Stopped {
            println!(
                "[kernel] Process {} Thread 1 is continued.",
                self.proc().pid()
            );
            drop(task);
            TASK_MANAGER.push(self);
        }
    }
}

impl TaskInner {
    pub fn task_state(&self) -> TaskState {
        self.task_state
    }

    pub fn task_state_mut(&mut self) -> &mut TaskState {
        &mut self.task_state
    }

    pub fn task_ctx(&self) -> &TaskContext {
        &self.task_ctx
    }

    pub fn task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.task_ctx
    }

    pub fn task_ctx_mut(&mut self) -> &mut TaskContext {
        &mut self.task_ctx
    }

    pub fn trap_ctx(&self) -> &TrapCtx {
        self.trap_ctx_handle.trap_ctx()
    }

    pub fn trap_ctx_mut(&mut self) -> &mut TrapCtx {
        self.trap_ctx_handle.trap_ctx_mut()
    }

    pub fn trap_ctx_ptr(&mut self) -> usize {
        self.trap_ctx_handle.trap_ctx_ptr()
    }

    pub fn trap_ctx_backup(&self) -> Option<&TrapCtx> {
        self.trap_ctx_backup.as_ref()
    }

    pub fn trap_ctx_backup_mut(&mut self) -> &mut Option<TrapCtx> {
        &mut self.trap_ctx_backup
    }

    pub fn sigs(&self) -> SignalFlags {
        self.sigs
    }

    pub fn sigs_mut(&mut self) -> &mut SignalFlags {
        &mut self.sigs
    }

    pub fn sig_mask(&self) -> SignalFlags {
        self.sig_mask
    }

    pub fn sig_mask_mut(&mut self) -> &mut SignalFlags {
        &mut self.sig_mask
    }

    pub fn user_stack(&self) -> &UserStack {
        &self.user_stack
    }

    pub fn page_table(&self) -> Arc<PageTable> {
        self.page_table.clone()
    }

    pub fn sig_handling(&self) -> Option<usize> {
        self.sig_handling
    }

    pub fn sig_handling_mut(&mut self) -> &mut Option<usize> {
        &mut self.sig_handling
    }

    pub fn exit_code(&self) -> isize {
        self.exit_code
    }

    pub fn tid(&self) -> usize {
        self.tid.id()
    }

    pub fn gid(&self) -> usize {
        self.gid.id()
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        println!(
            "Process {} thread {} is dropped.",
            self.proc().pid(),
            self.lock().tid()
        );
    }
}
