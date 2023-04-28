use core::cmp::max;

use crate::{
    config::{MIN_EXEC_TIME_SLICE, PELT_ATTENUATION, PELT_PERIOD},
    task::processor::Processor,
    time::get_time,
};

pub struct TaskTime {
    vruntime: usize,
    weight: usize,

    remaining: usize,
    pub last_restore: usize,

    // PELT related
    period: usize,
    history_load: usize,
    running_load: usize,
}

impl TaskTime {
    pub fn new(weight: usize) -> Self {
        let now = get_time();
        Self {
            vruntime: 0,
            weight,
            remaining: 0,
            last_restore: now,
            period: now / PELT_PERIOD,
            history_load: 0,
            running_load: 0,
        }
    }

    pub fn runout(&mut self) {
        self.vruntime += (self.remaining + self.weight - 1) / self.weight;
        self.remaining = 0;
        self.running_load = self.running_load.saturating_sub(MIN_EXEC_TIME_SLICE);
        // with some cost
    }

    pub fn setup(&mut self, rest_time: usize) {
        self.remaining = rest_time;
    }

    pub fn calibrate(&mut self, vruntime: usize) {
        self.vruntime = max(self.vruntime, vruntime);
    }

    pub fn clear(&mut self) {
        self.vruntime = 0;
    }

    pub fn trap(&mut self) {
        let now = get_time();
        let runtime = now - self.last_restore;
        self.remaining = self.remaining.saturating_sub(runtime);
        self.vruntime += (runtime + self.weight - 1) / self.weight;

        if pelt_period(now) == pelt_period(self.last_restore) {
            self.running_load += runtime;
        } else {
            if pelt_period(now) != pelt_period(self.last_restore) + 1 {
                warnln!(
                    "Process {} might run for too long.",
                    Processor::curr_proc().pid()
                );
            }
            self.history_load = self.history_load / PELT_ATTENUATION
                + (self.running_load + PELT_PERIOD * pelt_period(now) - self.last_restore);
            self.running_load = now % PELT_PERIOD;
            self.period = pelt_period(now);
        }
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

    pub fn history_load(&self) -> usize {
        self.history_load
    }
}

fn pelt_period(now: usize) -> usize {
    now / PELT_PERIOD
}
