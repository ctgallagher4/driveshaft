mod error;
mod job;
mod pool;
mod worker;

pub use crate::{
    error::DriveShaftError, pool::DriveShaftPool, pool::DriveShaftPoolBuilder, worker::WorkerType,
};

#[cfg(test)]
mod tests {

    use crate::{pool::DriveShaftPoolBuilder, worker::WorkerType};

    #[tokio::test]
    async fn test_create_worker() {
        let job = |ctx: &mut u8| {
            println!("Executed Job {}", ctx);
            ctx.clone()
        };

        let mut drive_shaft_pool = DriveShaftPoolBuilder::new()
            .worker_type(WorkerType::UnBound)
            .build(vec![1]);

        let _ = drive_shaft_pool.run_with(job).await;
    }
}
