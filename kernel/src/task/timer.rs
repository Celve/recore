use alloc::{
    collections::BinaryHeap,
    sync::{Arc, Weak},
};
use lazy_static::lazy_static;

use crate::sync::mcs::Mcs;

use super::task::Task;

pub struct Timer {
    tasks: Mcs<BinaryHeap<TimerUnit>>,
}

pub struct TimerUnit {
    time: usize,
    task: Weak<Task>,
}

lazy_static! {
    pub static ref TIMER: Timer = Timer::new();
}

impl Timer {
    pub fn new() -> Self {
        Self {
            tasks: Mcs::new(BinaryHeap::new()),
        }
    }
}

impl Timer {
    pub fn subscribe(&self, time: usize, task: &Arc<Task>) {
        self.tasks.lock().push(TimerUnit {
            time,
            task: task.phantom(),
        });
    }

    pub fn notify(&self, time: usize) {
        let mut top = self.tasks.lock().pop();
        while let Some(timer_unit) = top {
            if timer_unit.time > time {
                self.tasks.lock().push(timer_unit);
                break;
            }
            if let Some(task) = timer_unit.task.upgrade() {
                task.wakeup();
            }
            top = self.tasks.lock().pop();
        }
    }
}

impl PartialEq for TimerUnit {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for TimerUnit {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        other.time.partial_cmp(&self.time)
    }
}

impl Eq for TimerUnit {}

impl Ord for TimerUnit {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.time.cmp(&self.time)
    }
}
