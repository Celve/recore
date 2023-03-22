use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use riscv::register::sstatus::{self, SPP};

use crate::{
    config::TRAP_CONTEXT_START_ADDRESS,
    mm::{
        address::{PhyAddr, VirAddr},
        memory::{Memory, KERNEL_SPACE},
    },
    trap::{context::TrapContext, trampoline::restore, trap_handler},
};

use super::pid::{alloc_pid, Pid};
use crate::task::stack::KernelStack;

pub struct Task {
    pid: Pid,
    user_mem: Memory,
    task_status: TaskStatus,
    task_ctx: TaskContext,
    trap_ctx: PhyAddr, // raw pointer can't be shared between threads
    kernel_stack: KernelStack,
    parent: Option<Weak<Task>>,
    children: Vec<Arc<Task>>,
    exit_code: isize,
}

#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub sr: [usize; 12],
}

pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}

impl Task {
    pub fn from_elf(elf_data: &[u8], parent: Option<Weak<Task>>) -> Self {
        let pid = alloc_pid();
        let pid_int = pid.0;
        let (user_mem, user_sp, user_sepc) = Memory::from_elf(elf_data);
        let page_table = user_mem.page_table();

        let trap_ctx = page_table
            .translate(VirAddr::from(TRAP_CONTEXT_START_ADDRESS).floor_to_vir_page_num())
            .expect("[task] Unable to access trap context.")
            .get_ppn();
        println!(
            "[trap] Map {:#x} -> {:#x}",
            usize::from(VirAddr::from(TRAP_CONTEXT_START_ADDRESS)),
            usize::from(trap_ctx)
        );
        println!("[trap] User's sepc is {:#x}", user_sepc);

        let kernel_stack = KernelStack::new(pid_int);

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
            pid,
            user_mem,
            task_status: TaskStatus::Ready,
            task_ctx: TaskContext::new(restore as usize, kernel_stack.top().into()),
            trap_ctx: trap_ctx.into(),
            kernel_stack,
            parent,
            children: Vec::new(),
            exit_code: 0,
        }
    }

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

    pub fn exit_code_mut(&mut self) -> &mut isize {
        &mut self.exit_code
    }

    pub fn user_mem(&self) -> &Memory {
        &self.user_mem
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
