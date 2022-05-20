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
    E: Send,
{
    /// Helper method to auto-retry an async () -> Result.
    /// If it fails after `num_retries + 1` times, return the Err.
    async fn retry(&self, num_retries: u8) -> Result<T, E> {
        let mut last_err: Option<E> = None;
        for _ in 0..num_retries + 1 {
            let result = self().await;
            match result {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            };
        }
        Err(last_err.expect("definitely set"))
    }
}

impl<F, Fut, T, E> GraplAsyncRetry<Fut, T, E> for F
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>> + Send,
    E: Send,
{
}
