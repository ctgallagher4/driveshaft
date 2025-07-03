use crossbeam::deque::{Injector, Worker};
use std::{collections::VecDeque, fmt::Debug, sync::Arc};
use tokio::sync::oneshot;

use crate::{actor::Actor, error::DriveShaftError, job::Job};

pub struct DriveShaftPool<T> {
    injector: Arc<Injector<Job<T>>>,
    pub actors: Vec<Actor>,
}

impl<T> DriveShaftPool<T>
where
    T: Send + 'static,
{
    pub fn new(mut ctxs: VecDeque<T>) -> Self {
        let mut core_ids = VecDeque::new();
        let logical_cores = core_affinity::get_core_ids()
            .map(|v| {
                core_ids.extend(v.clone());
                v.len()
            })
            .unwrap_or(0);
        let num = ctxs.len();
        if num > logical_cores {
            eprintln!(
                "⚠️  [driveshaft] Warning: {num} threads requested but only {logical_cores} logical cores available. Consider lowering the thread count."
            );
        }

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
        for _ in 0..ctxs.len() {
            if let Some(worker) = workers.remove(0) {
                if let Some(ctx) = ctxs.remove(0) {
                    if let Some(core_id) = core_ids.remove(0) {
                        actors.push(Actor::new(
                            ctx,
                            worker,
                            Arc::clone(&injector),
                            Arc::clone(&stealers),
                            core_id,
                        ));
                    }
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
