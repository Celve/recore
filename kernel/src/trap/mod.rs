use core::arch::asm;

use fosix::syscall::SYSCALL_THREAD_CREATE;
use riscv::register::{scause, sip, utvec::TrapMode};

use crate::{
    config::TRAMPOLINE_ADDR,
    syscall::syscall,
    task::{processor::fetch_curr_task, suspend_yield},
};

use self::{signal::signal_handler, trampoline::restore};

pub mod context;
pub mod signal;
pub mod trampoline;

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_stvec();
    let trap = scause::read().cause();
    match trap {
        scause::Trap::Interrupt(intp) => match intp {
            scause::Interrupt::SupervisorSoft => {
                // acknowledge the software interrupt
                let sip = sip::read().bits();
                unsafe {
                    asm! {"csrw sip, {sip}", sip = in(reg) sip ^ 2};
                }
                suspend_yield();
            }
            scause::Interrupt::SupervisorTimer => todo!(),
            scause::Interrupt::SupervisorExternal => {
                println!("receive supervisor external interrupt");
            }
            scause::Interrupt::UserSoft => todo!(),
            scause::Interrupt::UserTimer => todo!(),
            scause::Interrupt::UserExternal => todo!(),
            scause::Interrupt::Unknown => todo!(),
            scause::Interrupt::VirtualSupervisorSoft => todo!(),
            scause::Interrupt::VirtualSupervisorTimer => todo!(),
            scause::Interrupt::VirtualSupervisorExternal => todo!(),
        },
        scause::Trap::Exception(excp) => match excp {
            scause::Exception::UserEnvCall => {
                let (id, args) = {
                    let task = fetch_curr_task();
                    let mut task_guard = task.lock();
                    let trap_ctx = task_guard.trap_ctx_mut();
                    trap_ctx.user_sepc += 4; // it must be added here
                    (
                        trap_ctx.saved_regs[17],
                        [
                            trap_ctx.saved_regs[10],
                            trap_ctx.saved_regs[11],
                            trap_ctx.saved_regs[12],
                        ],
                    )
                };
                let result = syscall(id, args);
                {
                    let task = fetch_curr_task();
                    let mut task_guard = task.lock();
                    *task_guard.trap_ctx_mut().a0_mut() = result as usize;
                    if id == SYSCALL_THREAD_CREATE {
                        println!("user_pc: {:#x}", task_guard.trap_ctx().user_sepc);
                    }
                }
            }
            scause::Exception::InstructionMisaligned => todo!(),
            scause::Exception::InstructionFault => todo!(),
            scause::Exception::IllegalInstruction => todo!(),
            scause::Exception::Breakpoint => todo!(),
            scause::Exception::LoadFault => todo!(),
            scause::Exception::StoreMisaligned => todo!(),
            scause::Exception::StoreFault => todo!(),
            scause::Exception::InstructionPageFault => todo!(),
            scause::Exception::LoadPageFault => todo!(),
            scause::Exception::StorePageFault => todo!(),
            scause::Exception::Unknown => todo!(),
            scause::Exception::VirtualSupervisorEnvCall => todo!(),
            scause::Exception::InstructionGuestPageFault => todo!(),
            scause::Exception::LoadGuestPageFault => todo!(),
            scause::Exception::VirtualInstruction => todo!(),
            scause::Exception::StoreGuestPageFault => todo!(),
        },
    }
    signal_handler();
    restore();
}

/// This is the trap handler for the supervisor mode.
///
/// It should be aligned to 4.
#[repr(align(4))]
pub fn fail() {
    panic!("[kernel] Get into trap when in supervisor mode.");
}

/// Set the trap handler to the `fail` function when trap occurs in the supervisor mode.
///
/// It has been proved that kernel would not trap in the supervisor mode when receiving the supervisor software interrupt.
/// It might due to the mechanism of the RISC-V.
pub fn set_kernel_stvec() {
    unsafe { riscv::register::stvec::write(fail as usize, TrapMode::Direct) };
}

pub fn set_user_stvec() {
    unsafe { riscv::register::stvec::write(TRAMPOLINE_ADDR, TrapMode::Direct) };
}
