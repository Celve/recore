use crate::task::processor::Processor;

pub fn sys_procdump() -> isize {
    Processor::procdump();
    0
}
