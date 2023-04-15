use fosix::signal::SignalFlags;

use crate::{
    config::NUM_SIGNAL,
    task::{
        cont, exit_yield,
        processor::{fetch_curr_proc, fetch_curr_task},
        stop_yield, suspend_yield,
        task::TaskState,
    },
};

pub fn signal_handler() {
    if fetch_curr_task().lock().sig_handling().is_none() {
        loop {
            let sigs = {
                let task = fetch_curr_task();
                let task_guard = task.lock();
                let proc = fetch_curr_proc();
                let proc_guard = proc.lock();
                let sig = task_guard.sigs();
                let sig_mask = task_guard.sig_mask();

                if let Some(sig_handling) = task_guard.sig_handling() {
                    sig & !sig_mask & !proc_guard.sig_actions()[sig_handling].mask()
                } else {
                    sig & !sig_mask
                }
            };

            for i in 0..NUM_SIGNAL {
                let sig = SignalFlags::from_bits(1 << i).unwrap();
                if sigs.contains(sig) {
                    println!("[kernel] Receive signal {}", i);
                    *fetch_curr_task().lock().sigs_mut() ^= sig;
                    if sig == SignalFlags::SIGKILL
                        || sig == SignalFlags::SIGSTOP
                        || sig == SignalFlags::SIGCONT
                    {
                        // signal is a kernel signal
                        kernel_signal_handler(i);
                    } else {
                        // signal is a user signal
                        user_signal_handler(i);
                        break;
                    }
                }
            }

            let status = fetch_curr_task().lock().task_state();
            if status != TaskState::Stopped {
                break;
            }

            suspend_yield();
        }
    }
}

fn kernel_signal_handler(sigid: usize) {
    let sig = SignalFlags::from_bits(1 << sigid).unwrap();
    match sig {
        SignalFlags::SIGKILL => exit_yield(-2),
        SignalFlags::SIGSTOP => stop_yield(),
        SignalFlags::SIGCONT => cont(),
        _ => {
            panic!("[kernel] Unhandled kernel signal.")
        }
    }
}

fn user_signal_handler(sigid: usize) {
    let handler = {
        let proc = fetch_curr_proc();
        let proc_guard = proc.lock();
        proc_guard.sig_actions()[sigid].handler()
    };

    if handler != 0 {
        let task = fetch_curr_task();
        let mut task_guard = task.lock();

        assert!(task_guard.trap_ctx_backup().is_none());
        *task_guard.trap_ctx_backup_mut() = Some(task_guard.trap_ctx().clone());
        *task_guard.sig_handling_mut() = Some(sigid);

        task_guard.trap_ctx_mut().user_sepc = handler;
        *task_guard.trap_ctx_mut().a0_mut() = sigid as usize;
    }
}
