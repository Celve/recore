#[repr(C)]
#[derive(Default)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub sr: [usize; 12],
}

impl TaskContext {
    pub fn new(ra: usize, sp: usize) -> Self {
        Self {
            ra,
            sp,
            sr: [0; 12],
        }
    }
}
