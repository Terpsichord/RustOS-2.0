use crate::{
    threads,
    threads::thread::{Thread, Tid},
};
use alloc::collections::VecDeque;
use hashbrown::HashMap;
use spin::{Lazy, Mutex, MutexGuard};
use x86_64::VirtAddr;

static SCHEDULER: Lazy<Mutex<Scheduler>> = Lazy::new(|| Mutex::new(Scheduler::new()));

#[derive(Default)]
pub struct Scheduler {
    current_thread: Option<Thread>,
    process_queue: VecDeque<Tid>,
    threads: HashMap<Tid, Thread>,
}

impl Scheduler {
    fn new() -> Self { Self::default() }

    fn get_next(&mut self) -> Option<&Thread> {
        self.process_queue
            .pop_front()
            .map(|tid| self.threads.get(&tid))
            .flatten()
    }

    pub unsafe fn schedule(&mut self, stack_address: VirtAddr) {
        let Some(mut current_thread) = self.current_thread.take() else {
            return;
        };
        if let Some(next_thread) = self.get_next() {
            unsafe {
                threads::context_switch(&mut current_thread.mut_context(), next_thread.context());
            }
        }
    }
}

pub fn get() -> MutexGuard<'static, Scheduler> { SCHEDULER.lock() }
