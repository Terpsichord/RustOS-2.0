use crate::task::TaskId;
use alloc::{sync::Arc, task::Wake};
use core::task::Waker;
use crossbeam_queue::ArrayQueue;

pub(super) struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    pub fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        Self {
            task_id,
            task_queue,
        }
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) { self.wake_by_ref(); }

    fn wake_by_ref(self: &Arc<Self>) {
        self.task_queue.push(self.task_id).expect("task queue full");
    }
}

impl From<TaskWaker> for Waker {
    fn from(value: TaskWaker) -> Self { Waker::from(Arc::new(value)) }
}
