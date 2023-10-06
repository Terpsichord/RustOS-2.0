use crate::threads::Context;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::VirtAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tid(u64);

impl Tid {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 { self.0 }
}

pub struct Thread {
    id: Tid,
    context: Context,
    stack_bounds: Option<StackBounds>,
}

impl Thread {
    pub fn context(&self) -> &Context { &self.context }

    pub fn mut_context(&mut self) -> &mut Context { &mut self.context }
}

pub struct StackBounds {
    start: VirtAddr,
    end: VirtAddr,
}

impl StackBounds {
    pub fn start(&self) -> VirtAddr { self.start }

    pub fn end(&self) -> VirtAddr { self.end }
}
