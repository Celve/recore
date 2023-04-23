use core::cmp::max;

use crate::time::get_time;

pub struct TaskTime {
    vruntime: usize,
    weight: usize,

    remaining: usize,
    pub last_restore: usize,
}

impl TaskTime {
    pub fn new(weight: usize) -> Self {
        Self {
            vruntime: 0,
            weight,
            remaining: 0,
            last_restore: get_time(),
        }
    }

    pub fn runout(&mut self) {
        self.vruntime += (self.remaining + self.weight - 1) / self.weight;
        self.remaining = 0;
    }

    pub fn setup(&mut self, rest_time: usize) {
        self.remaining = rest_time;
    }

    pub fn calibrate(&mut self, vruntime: usize) {
        self.vruntime = max(self.vruntime, vruntime);
    }

    pub fn trap(&mut self) {
        let runtime = get_time() - self.last_restore;
        self.remaining = self.remaining.saturating_sub(runtime);
        self.vruntime += (runtime + self.weight - 1) / self.weight;
    }

    pub fn restore(&mut self) {
        self.last_restore = get_time();
    }

    pub fn remaining(&self) -> usize {
        self.remaining
    }

    pub fn vruntime(&self) -> usize {
        self.vruntime
    }

    pub fn weight(&self) -> usize {
        self.weight
    }
}
