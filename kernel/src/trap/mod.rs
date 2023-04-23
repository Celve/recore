use core::arch::asm;

use riscv::register::{satp, scause, sepc, sip, stval, utvec::TrapMode};

use crate::{
    config::TRAMPOLINE_ADDR,
    drivers::{
        plic::{TargetPriority, PLIC},
        uart::UART,
    },
    fs::FUSE,
    syscall::syscall,
    task::processor::{fetch_curr_proc, fetch_curr_task, hart_id},
    time::get_time,
};

use self::{signal::signal_handler, trampoline::restore};

pub mod context;
pub mod signal;
pub mod trampoline;

/// The function that handles traps from user mode.
///
/// When the trap happens, the sie register would be set to 0.
/// Hence there is no supervisor mode interrupt or exception that could enter the trap handler again.
#[no_mangle]
pub fn trap_handler() -> ! {
    // Yielding should be done after all the traps are handled.
    // Because the scause is not maintained.
    fetch_curr_task().lock().task_time_mut().trap();

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

                fetch_curr_task().yield_now();
            }
            scause::Interrupt::SupervisorTimer => todo!(),
            scause::Interrupt::SupervisorExternal => {
                // acknowledge the external interrupt
                let sip = sip::read().bits();
                unsafe {
                    asm! {"csrw sip, {sip}", sip = in(reg) sip ^ (1 << 9)};
                }

                let id = PLIC.claim(hart_id(), TargetPriority::Supervisor);
                if id != 0 {
                    match id {
                        1 => FUSE.disk_manager().handle_irq(),
                        10 => UART.handle_irq(),
                        _ => panic!("Unknown interrupt id {}", id),
                    }
                    PLIC.complete(hart_id(), TargetPriority::Supervisor, id);

                    // let that task to be handled
                    fetch_curr_task().yield_now();
                }
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
#[no_mangle]
#[repr(align(4))]
pub fn fail() {
    println!(
        "[kernel] scause: {}, stval: {:#x}, sepc: {:#x}, satp: {:#x}",
        scause::read().bits(),
        stval::read(),
        sepc::read(),
        satp::read().bits(),
    );
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
