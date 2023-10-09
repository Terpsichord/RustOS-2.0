use crate::task::Task;
use alloc::rc::Rc;
use anyhow::{anyhow, Result};
use core::future::Future;
use crossbeam_queue::ArrayQueue;

#[derive(Clone)]
pub struct Spawner {
    spawned_tasks: Rc<ArrayQueue<Task>>,
}

impl Spawner {
    pub fn new(cap: usize) -> Self {
        Self {
            spawned_tasks: Rc::new(ArrayQueue::new(cap)),
        }
    }

    /// Spawn a new task.
    ///
    /// # Panics
    /// This function panics if the task queue is full; use [`try_spawn`] for a
    /// function that returns a `Result`.
    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        self.try_spawn(future).unwrap();
    }

    /// Tries to spawn a new task
    ///
    /// # Errors
    /// This function errors if the task queue is full, unlike [`spawn`] which
    /// panics
    pub fn try_spawn(&mut self, future: impl Future<Output = ()> + 'static) -> Result<()> {
        let task = Task::new(future);
        self.spawned_tasks
            .push(task)
            .map_err(|_| anyhow!("spawned task queue is full"))
    }

    pub fn tasks(&mut self) -> Tasks { Tasks(Rc::clone(&self.spawned_tasks)) }
}

pub struct Tasks(Rc<ArrayQueue<Task>>);

impl Iterator for Tasks {
    type Item = Task;

    fn next(&mut self) -> Option<Self::Item> { self.0.pop() }
}
