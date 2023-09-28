use crate::task::{waker::TaskWaker, Spawner, Task, TaskId};
use alloc::sync::Arc;
use core::task::{Context, Waker};
use crossbeam_queue::ArrayQueue;
use hashbrown::HashMap;

pub struct Executor {
    tasks: HashMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    spawner: Spawner,
    waker_cache: HashMap<TaskId, Waker>,
}

impl Executor {
    pub fn new(spawner: Spawner, cap: usize) -> Self {
        Self {
            tasks: HashMap::new(),
            task_queue: Arc::new(ArrayQueue::new(cap)),
            spawner,
            waker_cache: HashMap::new(),
        }
    }

    fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with same ID already exists");
        }
        self.task_queue.push(task_id).expect("task queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            for task in self.spawner.tasks() {
                self.spawn(task);
            }

            self.run_queued_tasks();
        }
    }

    fn run_queued_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                let waker = self
                    .waker_cache
                    .entry(task_id)
                    .or_insert_with(|| TaskWaker::new(task_id, Arc::clone(&self.task_queue)));

                let mut context = Context::from_waker(waker);

                if task.poll(&mut context).is_ready() {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
            }
        }
    }
}
