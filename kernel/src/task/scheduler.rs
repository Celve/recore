use core::cmp::max;

use alloc::{
    collections::BinaryHeap,
    sync::{Arc, Weak},
};

use crate::config::{MIN_AVG_TIME_SLICE, SCHED_PERIOD};

use super::task::Task;

pub struct Scheduler {
    tasks: BinaryHeap<SchedEntity>,
    period: usize,
    sum: usize,
}

pub struct SchedEntity {
    task: Weak<Task>,
    vruntime: usize,
    weight: usize,
}

impl Scheduler {
    /// Push the task to the scheduler, which would be locked for a while inside.
    pub fn push(&mut self, task: &Arc<Task>) {
        // calibrate with the saturating sub to avoid blocked task to be hold on for too long
        task.lock()
            .task_time_mut()
            .calibrate(self.calibration().saturating_sub(1));
        let sched_entity = task.to_sched_entity();
        self.sum += sched_entity.weight;
        self.tasks.push(sched_entity);
        self.period = max(self.tasks.len() * MIN_AVG_TIME_SLICE, SCHED_PERIOD);
    }

    pub fn pop(&mut self) -> Option<(Arc<Task>, usize)> {
        while let Some(item) = self.tasks.pop() {
            self.period = max(self.tasks.len() * MIN_AVG_TIME_SLICE, SCHED_PERIOD);
            self.sum -= item.weight;
            if let Some(task) = item.task.upgrade() {
                return Some((task, self.period * item.weight / (self.sum + item.weight)));
            }
        }
        None
    }

    pub fn peek(&mut self) -> Option<Arc<Task>> {
        while let Some(item) = self.tasks.peek() {
            if let Some(task) = item.task.upgrade() {
                return Some(task);
            }
            self.sum -= item.weight;
            self.tasks.pop();
        }
        None
    }

    pub fn calibration(&mut self) -> usize {
        if let Some(task) = self.peek() {
            task.lock().task_time().vruntime()
        } else {
            0
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            period: SCHED_PERIOD,
            sum: Default::default(),
        }
    }
}

impl SchedEntity {
    pub fn new(task: Weak<Task>, vruntime: usize, weight: usize) -> SchedEntity {
        SchedEntity {
            task,
            vruntime,
            weight,
        }
    }

    pub fn weight(&self) -> usize {
        self.weight
    }

    pub fn vruntime(&self) -> usize {
        self.vruntime
    }

    pub fn vruntime_mut(&mut self) -> &mut usize {
        &mut self.vruntime
    }
}

impl PartialEq for SchedEntity {
    fn eq(&self, other: &Self) -> bool {
        self.vruntime == other.vruntime
    }
}

impl Eq for SchedEntity {}

/// It's reversed because the binary heap is a max heap.
impl PartialOrd for SchedEntity {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        other.vruntime.partial_cmp(&self.vruntime)
    }
}

/// It's reversed because the binary heap is a max heap.
impl Ord for SchedEntity {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.vruntime.cmp(&self.vruntime)
    }
}
