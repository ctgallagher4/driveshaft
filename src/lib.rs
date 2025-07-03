mod actor;
mod error;
mod job;
mod pool;

pub use crate::{error::DriveShaftError, pool::DriveShaftPool};

#[cfg(test)]
mod tests {
    use crate::DriveShaftPool;

    #[tokio::test]
    async fn test_create_actor() {
        let job = |ctx: &mut u8| {
            println!("Executed Job {}", ctx);
            ctx.clone()
        };

        let ctxs = (0..10).collect();

        let drive_shaft_pool = DriveShaftPool::new(ctxs);

        let _ = drive_shaft_pool.run_with(job).await;
    }
}
