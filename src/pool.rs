use crossbeam::deque::{Injector, Worker};
use std::{collections::VecDeque, fmt::Debug, sync::Arc, thread::JoinHandle};
use tokio::sync::oneshot;

use crate::{actor::Actor, error::DriveShaftError, job::Job};

pub struct DriveShaftPool<T> {
    injector: Arc<Injector<Job<T>>>,
    actors: Vec<JoinHandle<()>>,
}

impl<T> DriveShaftPool<T>
where
    T: Send + 'static,
{
    pub fn new(mut ctxs: VecDeque<T>) -> Self {
        let injector = Arc::new(Injector::new());
        let mut workers = VecDeque::with_capacity(ctxs.len());
        let mut stealers = VecDeque::with_capacity(ctxs.len());

        ctxs.iter().for_each(|_ctx| {
            let worker = Worker::new_fifo();
            let stealer = worker.stealer();
            workers.push_back(worker);
            stealers.push_back(stealer);
        });

        let stealers = Arc::new(stealers);
        let mut actors = Vec::with_capacity(ctxs.len());
        for _i in 0..ctxs.len() {
            if let Some(worker) = workers.remove(0) {
                if let Some(ctx) = ctxs.remove(0) {
                    actors.push(Actor::new(
                        ctx,
                        worker,
                        Arc::clone(&injector),
                        Arc::clone(&stealers),
                    ));
                }
            }
        }

        Self { injector, actors }
    }

    pub async fn run_with<R, F>(&self, job: F) -> Result<R, DriveShaftError>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + Debug + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let wrapped_job = |context: &mut T| -> Result<(), DriveShaftError> {
            tx.send(job(context))
                .map_err(|_err| DriveShaftError::SendError)?;
            Ok(())
        };
        self.injector.push(Box::new(wrapped_job));

        rx.await.map_err(|_err| DriveShaftError::RecvError)
    }
}
