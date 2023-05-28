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
    trap::{
        context::{TrapCtx, TrapCtxHandle},
        trampoline::restore,
        trap_handler,
    },
};

use super::{
    context::TaskContext,
    processor::{Processor, PROCESSORS},
    scheduler::SchedEntity,
    time::TaskTime,
};

pub struct Task {
    proc: Weak<Proc>,
    inner: Spin<TaskInner>,
}

pub struct TaskInner {
    tid: Arc<Id>,
    gid: Arc<Id>,
    page_table: Arc<PageTable>,
    pub task_status: TaskStatus,
    task_ctx: TaskContext,
    trap_ctx_handle: TrapCtxHandle,
    pub trap_ctx_backup: Option<TrapCtx>,
    pub user_stack: UserStack,
    kernel_stack: KernelStack,
    pub exit_code: isize,
    pub sigs: SignalFlags,
    pub sig_mask: SignalFlags,
    pub sig_handling: Option<usize>,
    pub task_time: TaskTime,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
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
        weight: usize,
    ) -> Arc<Self> {
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

        Arc::new(Self {
            inner: Spin::new(TaskInner {
                tid,
                gid,
                page_table,
                task_status: TaskStatus::Running,
                task_ctx,
                trap_ctx_handle,
                trap_ctx_backup: None,
                user_stack,
                kernel_stack,
                exit_code: 0,
                sigs: SignalFlags::empty(),
                sig_mask: SignalFlags::empty(),
                sig_handling: None,
                task_time: TaskTime::new(weight),
            }),
            proc,
        })
    }

    pub fn renew(
        &self,
        tid: Arc<Id>,
        gid: Arc<Id>,
        proc: Weak<Proc>,
        page_table: Arc<PageTable>,
        weight: usize,
    ) -> Arc<Self> {
        assert_eq!(self.lock().tid.id(), tid.id());
        let kernel_stack = KernelStack::new(gid.id());
        let task_ctx = TaskContext::new(restore as usize, kernel_stack.top().into());
        let trap_ctx_handle = self.lock().trap_ctx_handle.renew(&page_table);
        let trap_ctx_backup = self.lock().trap_ctx_backup.clone();
        let user_stack = self.lock().user_stack.renew(&page_table);

        let trap_ctx = trap_ctx_handle.trap_ctx_mut();
        trap_ctx.kernel_sp = kernel_stack.top().into();

        Arc::new(Self {
            inner: Spin::new(TaskInner {
                tid,
                gid,
                page_table,
                task_status: TaskStatus::Running,
                task_ctx,
                trap_ctx_handle,
                trap_ctx_backup,
                user_stack,
                kernel_stack,
                exit_code: 0,
                sigs: SignalFlags::empty(), // FIX: inherit signal
                sig_mask: SignalFlags::empty(),
                sig_handling: None,
                task_time: TaskTime::new(weight),
            }),
            proc,
        })
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

    /// Restart the task.
    ///
    /// It's the companion method with `suspend()`.
    /// When the `suspend()` is called, the caller is reponsible to maintain the task elsewhere.
    /// Then the caller should wake up the task by calling this function, which would put the task into task manager again.
    pub fn wakeup(self: &Arc<Self>) {
        self.lock().task_status = TaskStatus::Running;
        Processor::curr_processor().lock().push_realtime(self);
    }

    pub fn exit(&self, exit_code: isize) {
        let mut guard = self.lock();
        guard.task_status = TaskStatus::Zombie;
        guard.exit_code = exit_code;
        infoln!(
            "Process {} thread {} exits with code {}.",
            self.proc().pid(),
            guard.tid(),
            exit_code
        );

        // in case that it's the main thread
        if guard.tid() == 1 {
            self.proc().exit(exit_code);
        }
    }
}

impl Task {
    /// Append the task's signal flags by locking it.
    pub fn kill(self: &Arc<Self>, sig: SignalFlags) {
        let mut task = self.lock();
        task.sigs |= sig;

        infoln!(
            "Process {} thread {} receives signal {}.",
            self.proc.upgrade().unwrap().pid(),
            task.tid(),
            sig.bits()
        );

        if sig.contains(SignalFlags::SIGCONT) && task.task_status == TaskStatus::Stopped {
            infoln!("Process {} thread 1 is continued.", self.proc().pid());
            drop(task);
            PROCESSORS[Processor::hart_id()].lock().push_normal(self);
        }
    }

    pub fn to_sched_entity(self: &Arc<Self>) -> SchedEntity {
        let task = self.lock();
        SchedEntity::new(
            self.phantom(),
            task.task_time.vruntime(),
            task.task_time.weight(),
            task.task_time.history_load(),
        )
    }
}

impl TaskInner {
    pub fn task_ctx_ptr(&mut self) -> *mut TaskContext {
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

    pub fn page_table(&self) -> Arc<PageTable> {
        self.page_table.clone()
    }

    pub fn tid(&self) -> usize {
        self.tid.id()
    }

    pub fn gid(&self) -> usize {
        self.gid.id()
    }
}
