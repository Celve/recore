use core::cmp::max;

use alloc::{
    collections::BinaryHeap,
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::config::{MIN_AVG_TIME_SLICE, SCHED_PERIOD};

use super::task::Task;

pub struct Scheduler {
    realtime: BinaryHeap<SchedEntity>,
    normal: BinaryHeap<SchedEntity>,
    period: usize,
    sum: usize,
    load: usize,
}

pub struct SchedEntity {
    task: Weak<Task>,
    vruntime: usize,
    weight: usize,
    load: usize,
}

impl Scheduler {
    /// Push the task to the scheduler, which would be locked for a while inside.
    pub fn push(&mut self, task: &Arc<Task>, is_realtime: bool) {
        // calibrate with the saturating sub to avoid blocked task to be hold on for too long
        task.lock()
            .task_time_mut()
            .calibrate(self.calibration().saturating_sub(1));

        let sched_entity = task.to_sched_entity();
        self.sum += sched_entity.weight;
        self.load += sched_entity.load;

        if is_realtime {
            self.realtime.push(sched_entity);
        } else {
            self.normal.push(sched_entity);
        }

        self.period = max(self.normal.len() * MIN_AVG_TIME_SLICE, SCHED_PERIOD);
    }

    fn calc_period(&mut self) {
        self.period = max(self.normal.len() * MIN_AVG_TIME_SLICE, SCHED_PERIOD);
    }

    fn pop_spec(&mut self, is_realtime: bool) -> Option<(Arc<Task>, usize)> {
        while let Some(sched_entity) = if is_realtime {
            self.realtime.pop()
        } else {
            self.normal.pop()
        } {
            self.sum -= sched_entity.weight;
            self.load -= sched_entity.load;
            self.calc_period();
            if let Some(task) = sched_entity.task.upgrade() {
                return Some((
                    task,
                    self.period * sched_entity.weight / (self.sum + sched_entity.weight),
                ));
            }
        }
        None
    }

    pub fn pop(&mut self) -> Option<(Arc<Task>, usize)> {
        let realtime_task = self.pop_spec(true);
        if realtime_task.is_some() {
            realtime_task
        } else {
            self.pop_spec(false)
        }
    }

    fn peek_spec(&mut self, is_realtime: bool) -> Option<Arc<Task>> {
        while let Some(sched_entity) = if is_realtime {
            self.realtime.peek()
        } else {
            self.normal.peek()
        } {
            if let Some(task) = sched_entity.task.upgrade() {
                return Some(task);
            }

            self.sum -= sched_entity.weight;
            self.load -= sched_entity.load;
            if is_realtime {
                self.realtime.pop();
            } else {
                self.normal.pop();
            }
            self.calc_period();
        }
        None
    }

    pub fn peek(&mut self) -> Option<Arc<Task>> {
        let realtime_task = self.peek_spec(true);
        if realtime_task.is_some() {
            realtime_task
        } else {
            self.peek_spec(false)
        }
    }

    pub fn calibration(&mut self) -> usize {
        if let Some(task) = self.peek() {
            task.lock().task_time().vruntime()
        } else {
            0
        }
    }

    pub fn load(&self) -> usize {
        self.load
    }

    pub fn len(&self) -> usize {
        self.normal.len()
    }

    pub fn iter(&self) -> alloc::collections::binary_heap::Iter<SchedEntity> {
        self.normal.iter()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            realtime: Default::default(),
            normal: Default::default(),
            period: SCHED_PERIOD,
            sum: 0,
            load: 0,
        }
    }
}

impl SchedEntity {
    pub fn new(task: Weak<Task>, vruntime: usize, weight: usize, load: usize) -> SchedEntity {
        SchedEntity {
            task,
            vruntime,
            weight,
            load,
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

    pub fn task(&self) -> Weak<Task> {
        self.task.clone()
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
