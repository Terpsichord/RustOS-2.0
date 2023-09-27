use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;
pub mod keyboard;
mod waker;

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

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static ID_COUNT: AtomicU64 = AtomicU64::new(0);
        TaskId(ID_COUNT.fetch_add(1, Ordering::Relaxed))
    }
}
