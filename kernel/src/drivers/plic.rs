use lazy_static::lazy_static;

use crate::config::VIRT_PLIC_ADDR;

pub struct Plic {
    base: usize,
}

pub enum TargetPriority {
    Machine = 0,
    Supervisor = 1,
}

lazy_static! {
    pub static ref PLIC: Plic = Plic::new(VIRT_PLIC_ADDR);
}

impl Plic {
    pub fn set_priority(&self, src_id: usize, priority: u32) {
        assert!(priority < 8);
        unsafe {
            self.priority_ptr(src_id).write_volatile(priority);
        }
    }

    pub fn get_priority(&self, src_id: usize) -> u32 {
        unsafe { self.priority_ptr(src_id).read_volatile() }
    }

    pub fn enable(&self, hart_id: usize, target_priority: TargetPriority, src_id: usize) {
        let (ptr, offset) = self.enable_ptr(hart_id, target_priority, src_id);
        unsafe {
            ptr.write_volatile(ptr.read_volatile() | (1 << offset));
        }
    }

    pub fn disable(&self, hart_id: usize, target_priority: TargetPriority, src_id: usize) {
        let (ptr, offset) = self.enable_ptr(hart_id, target_priority, src_id);
        unsafe {
            ptr.write_volatile(ptr.read_volatile() & !(1 << offset));
        }
    }

    pub fn set_threshold(&self, hart_id: usize, target_priority: TargetPriority, threshold: u32) {
        assert!(threshold < 8);
        unsafe {
            self.threshold_ptr(hart_id, target_priority)
                .write_volatile(threshold);
        }
    }

    pub fn get_threshold(&self, hart_id: usize, target_priority: TargetPriority) -> u32 {
        unsafe { self.threshold_ptr(hart_id, target_priority).read_volatile() }
    }

    pub fn claim(&self, hart_id: usize, target_priority: TargetPriority) -> usize {
        unsafe { self.claim_ptr(hart_id, target_priority).read_volatile() as usize }
    }

    pub fn complete(&self, hart_id: usize, target_priority: TargetPriority, src_id: usize) {
        unsafe {
            self.complete_ptr(hart_id, target_priority)
                .write_volatile(src_id as u32);
        }
    }
}

impl Plic {
    /// Fetch the priority pointer of each interrupt source.
    /// It defines the interrupt
    fn priority_ptr(&self, src_id: usize) -> *mut u32 {
        assert!(src_id > 0 && src_id <= 132);
        (self.base + src_id * 4) as *mut u32
    }

    /// Convert the hart_id and target_priority to the target_id.
    ///
    /// In other words, target_id is the combination of hart_id and target_priority.
    fn target_id(hart_id: usize, target_priority: TargetPriority) -> usize {
        let priority_num = TargetPriority::supported_num();
        hart_id * priority_num + target_priority as usize
    }

    /// Fetch the enable pointer of each interrupt target,
    /// which is correlated with the interrupt source and interrupt target.
    ///
    /// It represents that whether the interrupt from the specified source is enabled for the specified target.
    ///
    /// Because we only need one bit in the `u32`, therefore we return the `u32` and the offset inside it.
    fn enable_ptr(
        &self,
        hart_id: usize,
        target_priority: TargetPriority,
        src_id: usize,
    ) -> (*mut u32, usize) {
        let target_id = Plic::target_id(hart_id, target_priority);
        let (reg_id, reg_offset) = (src_id / 32, src_id % 32);
        (
            (self.base + 0x2000 + 0x80 * target_id + 0x4 * reg_id) as *mut u32,
            reg_offset,
        )
    }

    /// Fetch the pointer of the threshold of each interrupt target.
    fn threshold_ptr(&self, hart_id: usize, target_priority: TargetPriority) -> *mut u32 {
        let target_id = Plic::target_id(hart_id, target_priority);
        (self.base + 0x200000 + 0x1000 * target_id) as *mut u32
    }

    /// Fetch the claim pointer of each interrupt target.
    ///
    /// It's the same as the complete pointer.
    fn claim_ptr(&self, hart_id: usize, target_priority: TargetPriority) -> *mut u32 {
        let target_id = Plic::target_id(hart_id, target_priority);
        (self.base + 0x200004 + 0x1000 * target_id) as *mut u32
    }

    /// Fetch the complete pointer of each interrupt target.
    ///
    /// It's the same as the claim pointer.
    fn complete_ptr(&self, hart_id: usize, target_priority: TargetPriority) -> *mut u32 {
        let target_id = Plic::target_id(hart_id, target_priority);
        (self.base + 0x200004 + 0x1000 * target_id) as *mut u32
    }
}

impl Plic {
    pub fn new(base: usize) -> Self {
        Self { base }
    }
}

impl TargetPriority {
    pub fn supported_num() -> usize {
        2
    }
}
