pub use crate::task::executor::Executor;
use crate::task::spawner::Spawner;
use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;
pub mod keyboard;
pub mod spawner;
mod waker;

const QUEUE_CAPACITY: usize = 100;

pub fn init_executor_and_spawner() -> (Executor, Spawner) {
    let spawner = Spawner::new(QUEUE_CAPACITY);
    let executor = Executor::new(spawner.clone(), QUEUE_CAPACITY);
    (executor, spawner)
}

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, context: &mut Context<'_>) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static ID_COUNT: AtomicU64 = AtomicU64::new(0);
        TaskId(ID_COUNT.fetch_add(1, Ordering::Relaxed))
    }
}
