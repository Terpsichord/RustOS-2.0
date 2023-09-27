use crate::task::{waker::TaskWaker, Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with same ID already exists");
        }
        self.task_queue.push(task_id).expect("task queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_queued_tasks();
        }
    }

    fn run_queued_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            let Some(task) = self.tasks.get_mut(&task_id) else {
                continue;
            };
            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, self.task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}
