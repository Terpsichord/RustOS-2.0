use crate::threads::ThreadId;
use x86_64::VirtAddr;

pub struct Thread {
    id: ThreadId,
    stack_pointer: Option<VirtAddr>,
    stack_bounds: Option<StackBounds>,
}

pub struct StackBounds {
    start: VirtAddr,
    end: VirtAddr,
}

impl StackBounds {
    pub fn start(&self) -> VirtAddr { self.start }

    pub fn end(&self) -> VirtAddr { self.end }
}
