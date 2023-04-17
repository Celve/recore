use fosix::signal::SignalFlags;

use crate::{
    config::NUM_SIGNAL,
    task::{
        exit_yield,
        processor::{fetch_curr_proc, fetch_curr_task},
        suspend_yield,
        task::TaskState,
    },
};

/// The handler that handles all signals.
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

                        // break to do the action in user mode
                        break;
                    }
                }
            }

            let status = fetch_curr_task().lock().task_state();
            if status != TaskState::Stopped {
                break;
            }

            // for stop, it yields here
            suspend_yield();
        }
    }
}

/// The handler that handles all kernel signals, which should be delegated by the `signal_handler()`.
fn kernel_signal_handler(sigid: usize) {
    println!(
        "kernel signal handler is now handling {} with pid {} and tid {}",
        sigid,
        fetch_curr_proc().pid(),
        fetch_curr_task().lock().tid()
    );
    let sig = SignalFlags::from_bits(1 << sigid).unwrap();
    match sig {
        SignalFlags::SIGKILL => exit_yield(-2), // yield immediately
        SignalFlags::SIGSTOP => fetch_curr_task().stop(), // do not yield immediately
        SignalFlags::SIGCONT => fetch_curr_task().cont(),
        _ => {
            panic!("[kernel] Unhandled kernel signal.")
        }
    }
}

/// The handler that handles all user signals, which should be delegated by the `signal_handler()`.
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
