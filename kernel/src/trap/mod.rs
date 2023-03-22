use riscv::register::scause;

use crate::{syscall::syscall, task::manager::fetch_curr_task};

pub mod context;
pub mod trampoline;

pub fn trap_handler() {
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
                let curr_task_ptr = fetch_curr_task();
                let curr_task = curr_task_ptr.borrow_mut();
                let curr_trap_ctx = curr_task.trap_ctx_mut();
                let id = curr_trap_ctx.saved_regs[17];
                let arg1 = curr_trap_ctx.saved_regs[10];
                let arg2 = curr_trap_ctx.saved_regs[11];
                let arg3 = curr_trap_ctx.saved_regs[12];

                // move it on
                curr_trap_ctx.user_sepc += 4;

                drop(curr_trap_ctx);
                drop(curr_task);
                drop(curr_task_ptr);

                syscall(id, [arg1, arg2, arg3]);
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
}
