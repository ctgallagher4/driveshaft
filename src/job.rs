use crate::error::DriveShaftError;

pub type Job<T> = Box<dyn FnOnce(&mut T) -> Result<(), DriveShaftError> + Send + 'static>;
