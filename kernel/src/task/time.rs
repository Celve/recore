use core::cmp::max;

use crate::{
    config::{PELT_ATTENUATION, PELT_PERIOD},
    time::get_time,
};

pub struct TaskTime {
    vruntime: usize,
    weight: usize,

    remaining: usize,
    pub last_restore: usize,

    // PELT related
    period: usize,
    load: usize,
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
            load: 0,
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
        let now = get_time();
        let runtime = now - self.last_restore;
        self.remaining = self.remaining.saturating_sub(runtime);
        self.vruntime += (runtime + self.weight - 1) / self.weight;

        if pelt_period(now) == pelt_period(self.last_restore) {
            self.load += runtime;
        } else {
            if pelt_period(now) != pelt_period(self.last_restore) + 1 {
                println!("[kernel] One task might run for too long");
            }
            self.load = now % PELT_PERIOD + self.load / PELT_ATTENUATION;
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

    pub fn load(&self) -> usize {
        self.load
    }
}

fn pelt_period(now: usize) -> usize {
    now / PELT_PERIOD
}
