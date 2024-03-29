use alloc::{sync::Arc, vec::Vec};
use fosix::{
    fs::OpenFlags,
    signal::{SignalAction, SignalFlags, SIGCONT, SIGKILL, SIGSTOP},
};

use crate::{
    proc::{lockable::Lockable, manager::PROC_MANAGER, proc::ProcStatus},
    sync::{
        basic::{BlockLock, SpinLock},
        observable::Observable,
        semaphore::Semaphore,
    },
    task::processor::{Processor, PROCESSORS},
};

use super::{open_file, parse_str};

pub fn sys_exit(exit_code: isize) -> isize {
    Processor::exit(exit_code);
    0
}

pub fn sys_yield() -> isize {
    Processor::curr_task().lock().task_time.runout();
    0
}

pub fn sys_fork() -> isize {
    let proc = Processor::curr_proc().fork();
    let pid = proc.pid();
    let task = proc.lock().main_task();
    *task.lock().trap_ctx_mut().a0_mut() = 0;
    PROC_MANAGER.push(&proc);
    PROCESSORS[Processor::hart_id()].lock().push_normal(&task);
    infoln!("Fork a new process {}.", pid);
    pid as isize
}

pub fn sys_exec(path: usize, mut args_ptr: *const usize) -> isize {
    let name = unsafe { parse_str(path.into()) };
    let cwd = Processor::curr_proc().lock().cwd();
    let file = open_file(cwd, &name, OpenFlags::RDONLY);
    if let Some(file) = file {
        infoln!(
            "Process {} executes {}.",
            Processor::curr_proc().pid(),
            name
        );

        // parse args
        let mut args = Vec::new();
        loop {
            let arg = {
                let page_table = Processor::curr_proc().lock().page_table();
                unsafe { page_table.translate_any::<usize>((args_ptr as usize).into()) }
            };
            if *arg == 0 {
                break;
            }
            let mut str = unsafe { parse_str((*arg).into()) };
            str.push('\0');
            args.push(str);
            args_ptr = unsafe { args_ptr.add(1) };
        }

        Processor::curr_proc().exec(file, &args);
        args.len() as isize // otherwise it would be overrided
    } else {
        warnln!(
            "Process {} fails to execute {}.",
            Processor::curr_proc().pid(),
            name
        );
        -1
    }
}

pub fn sys_getpid() -> isize {
    Processor::curr_proc().pid() as isize
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();

    // find satisfied children
    let result = proc_guard.children.iter().position(|child| {
        (pid == -1 || pid as usize == child.pid()) && child.lock().proc_status == ProcStatus::Zombie
    });

    return if let Some(pos) = result {
        let removed_proc = proc_guard.children.remove(pos);
        unsafe {
            *proc_guard
                .page_table()
                .translate_any::<isize>(exit_code_ptr.into()) = removed_proc.lock().exit_code;
        }
        removed_proc.pid() as isize
    } else if proc_guard
        .children
        .iter()
        .any(|task| pid == -1 || task.pid() == pid as usize)
    {
        -2
    } else {
        -1
    };
}

#[no_mangle]
pub fn sys_sigreturn() -> isize {
    let task = Processor::curr_task();
    let mut task_guard = task.lock();
    task_guard.sig_handling.take();
    *task_guard.trap_ctx_mut() = task_guard.trap_ctx_backup.take().unwrap();
    task_guard.trap_ctx().a0() as isize
}

pub fn sys_kill(pid: usize, sig: usize) -> isize {
    let target = PROC_MANAGER.get(pid);
    if let Some(proc) = target {
        proc.kill(SignalFlags::from_bits(1 << sig).unwrap());
        0
    } else {
        -1
    }
}

pub fn sys_sigaction(sig_id: usize, new_action_ptr: usize, old_action_ptr: usize) -> isize {
    if new_action_ptr == 0
        || old_action_ptr == 0
        || sig_id == SIGKILL as usize
        || sig_id == SIGSTOP as usize
        || sig_id == SIGCONT as usize
    {
        return -1;
    }

    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    let page_table = proc_guard.page_table();
    let new_action = unsafe { page_table.translate_any::<SignalAction>(new_action_ptr.into()) };
    let old_action = unsafe { page_table.translate_any::<SignalAction>(old_action_ptr.into()) };

    *old_action = proc_guard.sig_actions[sig_id];
    proc_guard.sig_actions[sig_id] = *new_action;
    0
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    let task = Processor::curr_task();
    let mut task_guard = task.lock();
    if let Some(mask) = SignalFlags::from_bits(mask) {
        let old_mask = task_guard.sig_mask;
        task_guard.sig_mask = mask;
        old_mask.bits() as isize
    } else {
        -1
    }
}

pub fn sys_mutex_create(blocked: bool) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    proc_guard.lock_table.alloc(Arc::new(if blocked {
        Lockable::BlockMutex(BlockLock::new())
    } else {
        Lockable::SpinMutex(SpinLock::new())
    })) as isize
}

pub fn sys_mutex_lock(id: usize) -> isize {
    let lock = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.lock_table.get(id)
    };
    if let Some(lock) = lock {
        lock.lock();
        0
    } else {
        -1
    }
}

pub fn sys_mutex_unlock(id: usize) -> isize {
    let lock = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.lock_table.get(id)
    };
    if let Some(lock) = lock {
        lock.unlock();
        0
    } else {
        -1
    }
}

pub fn sys_semaphore_create(counter: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    proc_guard
        .sema_table
        .alloc(Arc::new(Semaphore::new(counter))) as isize
}

pub fn sys_semaphore_down(id: usize) -> isize {
    let sema = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.sema_table.get(id)
    };
    if let Some(sema) = sema {
        sema.down();
        0
    } else {
        -1
    }
}

pub fn sys_semaphore_up(id: usize) -> isize {
    let sema = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.sema_table.get(id)
    };
    if let Some(sema) = sema {
        sema.up();
        0
    } else {
        -1
    }
}

pub fn sys_condvar_create() -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    proc_guard.condvar_table.alloc(Arc::new(Observable::new())) as isize
}

pub fn sys_condvar_wait(condvar_id: usize, lock_id: usize) -> isize {
    let condvar = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.condvar_table.get(condvar_id)
    };
    let lock = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.lock_table.get(lock_id)
    };
    let task = Processor::curr_task();
    if let (Some(condvar), Some(lock)) = (condvar, lock) {
        if lock.is_locked() {
            lock.unlock();
            condvar.wait(task);
            lock.lock();
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_condvar_notify_one(id: usize) -> isize {
    let condvar = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.condvar_table.get(id)
    };
    if let Some(condvar) = condvar {
        condvar.notify_one();
        0
    } else {
        -1
    }
}

pub fn sys_condvar_notify_all(id: usize) -> isize {
    let condvar = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        proc_guard.condvar_table.get(id)
    };
    if let Some(condvar) = condvar {
        condvar.notify_all();
        0
    } else {
        -1
    }
}
