use std::time::Duration;

use futures::{
    TryFuture,
    TryFutureExt,
};
use futures_retry::{
    ErrorHandler,
    FutureFactory,
    FutureRetry,
    RetryPolicy,
};

pub struct ExponentialBackoffRetryHandler {
    retries: u32,
    duration: Duration,
}
impl<T> ErrorHandler<T> for ExponentialBackoffRetryHandler
where
    T: Sized,
{
    type OutError = T;

    fn handle(&mut self, attempt: usize, e: T) -> RetryPolicy<Self::OutError> {
        let attempt: u32 = attempt
            .try_into()
            .expect("You shouldn't be retrying >u32 times");
        if attempt < self.retries {
            let exponent: u32 = u32::pow(2, attempt);
            RetryPolicy::WaitRetry(self.duration * exponent)
        } else {
            RetryPolicy::ForwardError(e)
        }
    }
}
impl Default for ExponentialBackoffRetryHandler {
    fn default() -> Self {
        Self {
            retries: 3,
            duration: Duration::from_secs(1),
        }
    }
}

/// A simple wrapper around a Future to do exponential backoff retries.
pub async fn simple_exponential_backoff_retry<F, I>(factory: F) -> Result<I::Ok, I::Error>
where
    F: FutureFactory<FutureItem = I>,
    I: TryFuture,
{
    let exp_retry_handler = ExponentialBackoffRetryHandler::default();
    FutureRetry::new(factory, exp_retry_handler)
        .map_ok(|(response, _)| response)
        .map_err(|(err, _)| err)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_retry_works_correctly() {
        let mut handler = ExponentialBackoffRetryHandler {
            retries: 3,
            duration: Duration::from_secs(1),
        };
        let some_error = "hey";

        let policy = handler.handle(0, some_error);
        assert_eq!(policy, RetryPolicy::WaitRetry(Duration::from_secs(1)));

        let policy = handler.handle(1, some_error);
        assert_eq!(policy, RetryPolicy::WaitRetry(Duration::from_secs(2)));

        let policy = handler.handle(2, some_error);
        assert_eq!(policy, RetryPolicy::WaitRetry(Duration::from_secs(4)));

        let policy = handler.handle(3, some_error);
        assert_eq!(policy, RetryPolicy::ForwardError(some_error));
    }
}
