use crossbeam::channel::{Receiver, Sender};

use crate::error::DriveShaftError;

pub type Job<T> = Box<dyn FnOnce(&mut T) -> Result<(), DriveShaftError> + Send + 'static>;
pub type JobReciever<T> = Receiver<Job<T>>;
pub type JobSender<T> = Sender<Job<T>>;
