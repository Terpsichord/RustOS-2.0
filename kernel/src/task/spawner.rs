use crate::task::Task;
use alloc::sync::Arc;
use core::future::Future;
use crossbeam_queue::ArrayQueue;

#[derive(Clone)]
pub struct Spawner {
    spawned_tasks: Arc<ArrayQueue<Task>>,
}

impl Spawner {
    pub fn new(cap: usize) -> Self {
        Self {
            spawned_tasks: Arc::new(ArrayQueue::new(cap)),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let task = Task::new(future);
        self.spawned_tasks
            .push(task)
            .unwrap_or_else(|_| panic!("spawned task queue full"))
    }

    pub fn tasks(&mut self) -> Tasks { Tasks(Arc::clone(&self.spawned_tasks)) }
}

pub struct Tasks(Arc<ArrayQueue<Task>>);

impl Iterator for Tasks {
    type Item = Task;

    fn next(&mut self) -> Option<Self::Item> { self.0.pop() }
}
