use thiserror::Error;

#[derive(Debug, Error)]
pub enum DriveShaftError {
    #[error("failed to send job to worker thread: channel is closed or full")]
    SendError,

    #[error("worker thread terminated before sending result")]
    RecvError,

    #[error("no available workers in pool")]
    NoWorkers,
}
