#![allow(unused)]
pub mod executor;
pub mod keyboard;
pub mod reactor;

use core::{future::Future, pin::Pin};
use crossbeam::queue::ArrayQueue;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        TaskId(ID.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(fut: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::pin(fut),
        }
    }

    fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
}
