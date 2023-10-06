use core::sync::atomic::{AtomicU64, Ordering};

pub struct Pid(u64);

impl Pid {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Pid(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

struct Process {
    id: Pid,
}
// impl Process {
//     pub fn new() -> Self { Process { id: Pid::new() } }
//
//     pub unsafe fn new_kernel() { unsafe {
// scheduler::get().schedule(Self::new()); } } }
