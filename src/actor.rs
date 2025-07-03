use core_affinity::CoreId;
use crossbeam::deque::{Injector, Stealer, Worker};
use std::{collections::VecDeque, iter, sync::Arc, thread::JoinHandle};

use crate::job::Job;

pub struct Actor {
    pub handle: JoinHandle<()>,
}

impl Actor {
    pub fn new<T: Send + 'static>(
        mut ctx: T,
        fifo: Worker<Job<T>>,
        global: Arc<Injector<Job<T>>>,
        stealers: Arc<VecDeque<Stealer<Job<T>>>>,
        core_id: CoreId,
    ) -> Self {
        let handle = std::thread::spawn(move || {
            core_affinity::set_for_current(core_id);
            loop {
                let job = fifo.pop().or_else(|| {
                    iter::repeat_with(|| {
                        global
                            .steal_batch_and_pop(&fifo)
                            .or_else(|| stealers.iter().map(|s| s.steal()).collect())
                    })
                    .find(|s| !s.is_retry())
                    .and_then(|s| s.success())
                });
                if let Some(job) = job {
                    let _ = job(&mut ctx);
                }
            }
        });
        Self { handle }
    }
}
