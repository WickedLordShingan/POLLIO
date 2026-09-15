use super::{Task, TaskId};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
    task::Wake,
};
extern crate alloc;
use alloc::collections::VecDeque;
use crossbeam::queue::ArrayQueue;

pub struct Executor {
    taskqueue: Arc<ArrayQueue<TaskId>>,
    id_task_map: BTreeMap<TaskId, Task>,
    executor_handle: std::thread::Thread,
}

//make thread safe and create a static for executor
impl Executor {
    pub fn init() -> Self {
        Executor {
            taskqueue: Arc::from(ArrayQueue::new(128)),
            id_task_map: BTreeMap::new(),
            executor_handle: std::thread::current(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let Self {
            taskqueue,
            id_task_map,
            executor_handle,
        } = self;
        let task = Task::new(future);
        let task_id = TaskId::new();
        if self.id_task_map.insert(task_id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        taskqueue.push(task_id);
    }

    fn run_ready_task(&mut self) {
        while let Some(id) = self.taskqueue.pop() {
            if let Some(mut task) = self.id_task_map.get_mut(&id) {
                let task_waker =
                    TaskWaker::new(id, self.taskqueue.clone(), self.executor_handle.clone());
                let waker = Waker::from(Arc::from(task_waker));
                let mut cx = Context::from_waker(&waker);

                match task.poll(&mut cx) {
                    Poll::Ready(()) => {
                        self.id_task_map.remove(&id);
                    }
                    Poll::Pending => {}
                }
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_ready_task();
            std::thread::park();
        }
    }
}

use core::task::{self, Poll, Waker};
use std::task::Context;

struct TaskWaker {
    id: TaskId,
    taskqueue: Arc<ArrayQueue<TaskId>>,
    handle: std::thread::Thread,
}

impl TaskWaker {
    fn new(id: TaskId, taskqueue: Arc<ArrayQueue<TaskId>>, handle: std::thread::Thread) -> Self {
        Self {
            id,
            taskqueue,
            handle,
        }
    }

    fn wake_task(&self) {
        self.taskqueue.push(self.id).expect("QUEUE IS FULL");
        std::thread::Thread::unpark(&self.handle);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
