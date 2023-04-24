use spin::Spin;

use crate::task::processor::Processor;

use super::waiting_queue::WaitingQueue;

pub struct Semaphore {
    inner: Spin<SemaphoreInner>,
}

pub struct SemaphoreInner {
    counter: usize,
    queue: WaitingQueue,
}

impl Semaphore {
    pub fn new(counter: usize) -> Semaphore {
        Self {
            inner: Spin::new(SemaphoreInner {
                counter,
                queue: WaitingQueue::new(),
            }),
        }
    }

    pub fn down(&self) {
        let sema = &self.inner;
        while sema.lock().counter == 0 {
            {
                let task = Processor::curr_task();
                sema.lock().queue.push(&task);
            }
            Processor::suspend();
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
