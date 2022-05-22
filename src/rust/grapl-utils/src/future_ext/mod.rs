use std::{
    future::Future,
    time::Duration,
};

pub trait GraplFutureExt: Future {
    /// Helper method that creates a [`tokio::time::error::Elapsed`] future from the current future
    /// ```
    /// # use grapl_utils::future_ext::GraplFutureExt;
    /// # use std::time::Duration;
    /// # async fn _ignore() {
    /// let my_future = async {
    ///     123
    /// };
    ///
    /// my_future.timeout(Duration::from_secs(3)).await;
    /// # }
    /// ```
    fn timeout(self, duration: Duration) -> tokio::time::Timeout<Self>
    where
        Self: Sized,
    {
        tokio::time::timeout(duration, self)
    }
}

impl<T: ?Sized> GraplFutureExt for T where T: Future {}

#[async_trait::async_trait]
pub trait GraplAsyncRetry<Fut, T, E>: Fn() -> Fut
where
    Fut: Future<Output = Result<T, E>> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Helper method to auto-retry an async () -> Result.
    /// If it fails after `num_retries + 1` times, return the Err.
    async fn retry(&self, num_retries: u8) -> Result<T, E> {
        let sleep_duration = std::time::Duration::from_secs(1);
        for i in 0..num_retries + 1 {
            match self().await {
                Ok(t) => return Ok(t),
                Err(e) if i == num_retries => return Err(e),
                Err(_) => std::thread::sleep(sleep_duration),
            };
        }
        unreachable!("Loop should always return");
    }
}

impl<F, Fut, T, E> GraplAsyncRetry<Fut, T, E> for F
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
}
