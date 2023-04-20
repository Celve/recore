use spin::mutex::Mutex;

use crate::task::processor::fetch_curr_task;

use super::waiting_queue::WaitingQueue;

pub struct Semaphore {
    inner: Mutex<SemaphoreInner>,
}

pub struct SemaphoreInner {
    counter: usize,
    queue: WaitingQueue,
}

impl Semaphore {
    pub fn new(counter: usize) -> Semaphore {
        Self {
            inner: Mutex::new(SemaphoreInner {
                counter,
                queue: WaitingQueue::new(),
            }),
        }
    }

    pub fn down(&self) {
        let sema = &self.inner;
        while sema.lock().counter == 0 {
            let task = fetch_curr_task();
            sema.lock().queue.push(&task);
            task.suspend();
        }
        sema.lock().counter -= 1;
    }

    pub fn up(&self) {
        let sema = &self.inner;
        if sema.lock().counter == 0 {
            let task = sema.lock().queue.pop();
            if let Some(task) = task {
                task.wake_up();
            }
        }
        sema.lock().counter += 1;
    }
}
