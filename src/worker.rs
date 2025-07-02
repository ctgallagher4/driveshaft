use crossbeam::channel::{Receiver, Sender, bounded, unbounded};
use std::thread;

use crate::job::{Job, JobReciever, JobSender};

#[derive(Copy, Clone)]
pub enum WorkerType {
    Bound(usize),
    UnBound,
}

fn spawn_tx<T: Send + 'static>(mut context: T, worker_type: WorkerType) -> JobSender<T> {
    let (tx, rx): (JobSender<T>, JobReciever<T>) = match worker_type {
        WorkerType::Bound(size) => bounded::<Job<T>>(size),
        WorkerType::UnBound => unbounded::<Job<T>>(),
    };
    let _handle = thread::spawn(move || {
        while let Ok(func) = rx.recv() {
            let context_ref = &mut context;
            func(context_ref).expect("The user gave us a good closure!")
        }
    });
    tx
}

struct BoundedWorker<T> {
    sender: JobSender<T>,
}

impl<T> BoundedWorker<T> {
    fn new(context: T, size: usize) -> BoundedWorker<T>
    where
        T: Send + 'static,
    {
        let tx = spawn_tx(context, WorkerType::Bound(size));
        BoundedWorker { sender: tx }
    }
}

struct UnboundedWorker<T> {
    pub sender: JobSender<T>,
}

impl<T> UnboundedWorker<T> {
    fn new(context: T) -> UnboundedWorker<T>
    where
        T: Send + 'static,
    {
        let tx = spawn_tx(context, WorkerType::UnBound);
        UnboundedWorker { sender: tx }
    }
}

pub enum Worker<T> {
    Bounded(BoundedWorker<T>),
    Unbounded(UnboundedWorker<T>),
}

impl<T> Worker<T> {
    pub fn new(worker_type: WorkerType, context: T) -> Worker<T>
    where
        T: Send + 'static,
    {
        match worker_type {
            WorkerType::Bound(size) => Worker::Bounded(BoundedWorker::new(context, size)),
            WorkerType::UnBound => Worker::Unbounded(UnboundedWorker::new(context)),
        }
    }

    pub fn send(&self, job: Job<T>) {
        match self {
            Self::Bounded(worker) => worker.sender.send(job),
            Self::Unbounded(worker) => worker.sender.send(job),
        };
    }
}
