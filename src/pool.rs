use std::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

use tokio::sync::oneshot;

use crate::{
    error::DriveShaftError,
    worker::{Worker, WorkerType},
};

pub struct DriveShaftPoolBuilder {
    worker_type: Option<WorkerType>,
}

impl DriveShaftPoolBuilder {
    pub fn new() -> Self {
        Self { worker_type: None }
    }

    pub fn worker_type(self, worker_type: WorkerType) -> Self {
        Self {
            worker_type: Some(worker_type),
        }
    }

    pub fn build<T>(self, contexts: Vec<T>) -> DriveShaftPool<T>
    where
        T: Send + 'static,
    {
        let worker_type = match self.worker_type {
            None => WorkerType::UnBound,
            Some(type_of_worker) => type_of_worker,
        };

        DriveShaftPool {
            current_worker: AtomicUsize::new(0),
            workers: contexts
                .into_iter()
                .map(|ctx| Worker::new(worker_type, ctx))
                .collect(),
        }
    }
}

pub struct DriveShaftPool<T> {
    current_worker: AtomicUsize,
    workers: Vec<Worker<T>>,
}

impl<T> DriveShaftPool<T>
where
    T: Send + 'static,
{
    pub async fn run_with<R, F>(&mut self, job: F) -> Result<R, DriveShaftError>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + Debug + 'static,
    {
        let num_workers = self.workers.len();
        let idx = self.current_worker.fetch_add(1, Ordering::Relaxed) % num_workers;
        let (tx, rx) = oneshot::channel();

        let wrapped_job = |context: &mut T| -> Result<(), DriveShaftError> {
            tx.send(job(context))
                .map_err(|_err| DriveShaftError::SendError)?;
            Ok(())
        };
        self.workers[idx].send(Box::new(wrapped_job));
        rx.await.map_err(|_err| DriveShaftError::RecvError)
    }
}
