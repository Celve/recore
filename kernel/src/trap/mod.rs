use riscv::register::scause;

use crate::{syscall::syscall, task::processor::fetch_curr_task};

use self::trampoline::restore;

pub mod context;
pub mod trampoline;

#[no_mangle]
pub fn trap_handler() -> ! {
    let trap = scause::read().cause();
    match trap {
        scause::Trap::Interrupt(intp) => match intp {
            scause::Interrupt::SupervisorSoft => todo!(),
            scause::Interrupt::SupervisorTimer => todo!(),
            scause::Interrupt::SupervisorExternal => todo!(),
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
                    let task_guard = task.lock();
                    let trap_ctx = task_guard.trap_ctx_mut();
                    task_guard.trap_ctx_mut().user_sepc += 4; // it must be added here
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
                    let task_guard = task.lock();
                    *task_guard.trap_ctx_mut().a0_mut() = result as usize;
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
    restore();
}
