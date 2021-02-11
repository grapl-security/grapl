use std::future::Future;
use std::time::Duration;

pub trait GraplFutureExt: Future {
    /// Helper method that creates a [`tokio::time::Timeout`] future from the current future
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
