use std::{
    future::Future,
    iter::{
        IntoIterator,
        Iterator,
    },
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};
use std::num::NonZeroUsize;

use action::Action;
use pin_project::pin_project;
use recloser::{
    AnyError,
    AsyncRecloser,
    Recloser,
    RecloserBuilder,
    RecloserFuture,
};
use tokio::time::{
    error::Elapsed,
    sleep_until,
    timeout,
    Duration,
    Instant,
    Sleep,
    Timeout,
};
pub use tokio_retry::strategy;

pub mod action;

#[derive(thiserror::Error, Debug)]
pub enum Error<E: std::error::Error> {
    /// An error occurred while executing the underlying future
    #[error(transparent)]
    Inner(E),
    /// A rejected call is returned immediately when the circuit is opened
    #[error("Reject")]
    Rejected,
    /// A timeout for an underlying call has occurred
    #[error("Elapsed")]
    Elapsed(Elapsed),
}

#[pin_project(project = ExecuteStateProj)]
enum ExecuteState<A>
where
    A: Action,
{
    Running(#[pin] RecloserFuture<TryTimeout<<A as action::Action>::Future>, AnyError>),
    Sleeping(#[pin] Sleep),
}

impl<A: Action> ExecuteState<A> {
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> ExecuteFuturePoll<A> {
        match self.project() {
            ExecuteStateProj::Running(future) => match future.poll(cx) {
                Poll::Ready(Ok(p)) => ExecuteFuturePoll::Running(Poll::Ready(Ok(p))),
                Poll::Ready(Err(recloser::Error::Rejected)) => {
                    ExecuteFuturePoll::Running(Poll::Ready(Err(Error::Rejected)))
                }
                Poll::Ready(Err(recloser::Error::Inner(e))) => {
                    ExecuteFuturePoll::Running(Poll::Ready(Err(e)))
                }
                Poll::Pending => ExecuteFuturePoll::Running(Poll::Pending),
            },
            ExecuteStateProj::Sleeping(future) => ExecuteFuturePoll::Sleeping(future.poll(cx)),
        }
    }
}

enum ExecuteFuturePoll<A>
where
    A: Action,
{
    Running(Poll<ActionResult<A>>),
    Sleeping(Poll<()>),
}

pub type ActionResult<A> = Result<<A as Action>::Item, Error<<A as Action>::Error>>;

/// Internal helper Future that flattens timeout errors into crate::Error
#[pin_project]
struct TryTimeout<F>
where
    F: Future,
{
    #[pin]
    future: Timeout<F>,
    timeout: Duration,
}

impl<F> TryTimeout<F>
where
    F: Future,
{
    fn new(t: Duration, future: F) -> Self {
        TryTimeout {
            future: timeout(t, future),
            timeout: t,
        }
    }
}

impl<F, T, E> Future for TryTimeout<F>
where
    F: Future<Output = Result<T, E>>,
    E: std::error::Error,
{
    type Output = Result<T, Error<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Ready(Ok(Ok(p))) => Poll::Ready(Ok(p)),
            Poll::Ready(Ok(Err(e))) => Poll::Ready(Err(Error::Inner(e))),
            Poll::Ready(Err(e @ Elapsed { .. })) => Poll::Ready(Err(Error::Elapsed(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Future that drives multiple attempts at an action via a retry strategy.
/// All executions go through a circuit breaker.
#[pin_project]
pub struct Execution<'a, I, A>
where
    I: Iterator<Item = Duration>,
    A: Action,
{
    strategy: I,
    #[pin]
    state: ExecuteState<A>,
    recloser: &'a AsyncRecloser,
    timeout: Duration,
    action: A,
}

impl<'a, I, A> Execution<'a, I, A>
where
    I: Iterator<Item = Duration>,
    A: Action,
{
    fn attempt(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ActionResult<A>> {
        let future = {
            let this = self.as_mut().project();

            this.recloser
                .call(TryTimeout::new(this.timeout.clone(), this.action.run()))
        };
        self.as_mut()
            .project()
            .state
            .set(ExecuteState::Running(future));
        self.poll(cx)
    }

    fn retry(
        mut self: Pin<&mut Self>,
        err: Error<A::Error>,
        cx: &mut Context,
    ) -> Result<Poll<ActionResult<A>>, Error<<A as Action>::Error>> {
        match self.as_mut().project().strategy.next() {
            None => {
                tracing::debug!("No more retries");
                Err(err)
            }
            Some(duration) => {
                tracing::debug!("Retrying in {}ms", duration.as_millis());
                let deadline = Instant::now() + duration;
                let future = sleep_until(deadline);
                self.as_mut()
                    .project()
                    .state
                    .set(ExecuteState::Sleeping(future));
                Ok(self.poll(cx))
            }
        }
    }
}

impl<'a, I, A> Future for Execution<'a, I, A>
where
    I: Iterator<Item = Duration>,
    A: Action,
{
    type Output = ActionResult<A>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.as_mut().project().state.poll(cx) {
            ExecuteFuturePoll::Running(poll_result) => match poll_result {
                Poll::Ready(Ok(ok)) => Poll::Ready(Ok(ok)),
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(err)) => match self.retry(err, cx) {
                    Ok(poll) => poll,
                    Err(err) => Poll::Ready(Err(err)),
                },
            },
            ExecuteFuturePoll::Sleeping(poll_result) => match poll_result {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => self.attempt(cx),
            },
        }
    }
}

/// Based on https://resilience4j.readme.io/docs/circuitbreaker
pub struct ExecutorConfig {
    builder: RecloserBuilder,
    timeout: Duration,
}

impl ExecutorConfig {
    /// Creates a new `ExecutorConfig` from a `timeout`. Note that the `timeout`
    /// is *not* a global timeout, but will be applied to each individual call
    /// of the underlying future.
    /// The rest of the parameters - threshold, closed_len, etc, are initialized
    /// to defaults.
    pub fn new(timeout: Duration) -> Self {
        let builder = Recloser::custom()
            .error_rate(0.5)
            .closed_len(100)
            .half_open_len(10)
            .open_wait(Duration::from_secs(5));
        ExecutorConfig { builder, timeout }
    }

    /// The timeout for the individual executions of the future
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The threshold for opening the circuit, expressed as a float between 0 and 1.0.
    /// The state of the CircuitBreaker changes from CLOSED to OPEN when the failure rate
    /// is equal or greater than `threshold`.
    /// For example when more than 50% (`0.5f32`) of the recorded calls have failed.
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.builder = self.builder.error_rate(threshold);
        self
    }

    /// How many calls will the circuit remain closed for before the failure
    /// rate is recalculated and the circuit may open
    pub fn closed_len(mut self, closed_len: NonZeroUsize) -> Self {
        self.builder = self.builder.closed_len(closed_len.get());
        self
    }

    /// How many calls will the circuit remain HalfOpen for before the failure
    /// rate is recalculated and the circuit may either open or close
    pub fn half_open_len(mut self, half_open_len: NonZeroUsize) -> Self {
        self.builder = self.builder.half_open_len(half_open_len.get());
        self
    }

    /// The duration for which the circuit will remain Open, returning
    /// with immediate Rejected errors, before transitioning to HalfOpen
    pub fn open_wait(mut self, open_wait: Duration) -> Self {
        self.builder = self.builder.open_wait(open_wait);
        self
    }
}

/// Executor is a thread safe Circuit Breaker, Timeout, and Retry helper.
/// For a given future F,
/// 1. First the circuit breaker is checked
/// 2. If the circuit is open, the future is rejected with a Rejected error
/// 3. If the circuit is closed, the future is executed
/// 4. If the future does not complete within `timeout` `Error::Elapsed` is returned
///
/// `Executor` forces the caller to provide a timeout, and the circuit will open if too many
/// calls exceed this timeout. This should help servers under heavy load to recover.
///
/// When the circuit is OPEN, the future is rejected with `Error::Rejected`.
///
/// After a wait time duration has elapsed, the CircuitBreaker state changes from OPEN to HALF_OPEN
/// and permits `half_open_len` calls to see if the backend is still unavailable or has
/// become available again.
///
/// If the failure rate or slow call rate is then equal or greater than the configured threshold,
/// the state changes back to OPEN. If the failure rate and slow call rate is below the threshold,
/// the state changes back to CLOSED.
#[derive(Clone)]
pub struct Executor {
    recloser: AsyncRecloser,
    timeout: Duration,
}

impl Executor {
    pub fn new(config: ExecutorConfig) -> Self {
        let timeout = config.timeout;
        Executor {
            recloser: AsyncRecloser::from(config.builder.build()),
            timeout,
        }
    }

    pub fn spawn<A, I, T>(&self, strategy: T, mut action: A) -> Execution<I, A>
    where
        I: Iterator<Item = Duration>,
        A: Action,
        T: IntoIterator<IntoIter = I, Item = Duration>,
    {
        Execution {
            strategy: strategy.into_iter(),
            state: ExecuteState::Running(
                self.recloser
                    .call(TryTimeout::new(self.timeout.clone(), action.run())),
            ),
            timeout: self.timeout.clone(),
            recloser: &self.recloser,
            action,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::Ordering,
        Arc,
    };

    use super::*;

    #[derive(thiserror::Error, Debug)]
    enum MyCustomError {
        #[error("OhNo!")]
        OhNo(i32),
    }

    #[tokio::test]
    async fn test_retries() -> Result<(), Box<dyn std::error::Error>> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(10)));

        executor
            .spawn([], || async move { Ok::<(), MyCustomError>(()) })
            .await?;

        let i = Arc::new(std::sync::atomic::AtomicI32::new(0));
        executor
            .spawn([Duration::from_millis(0), Duration::from_millis(0)], || {
                let i = i.clone();
                async move {
                    let i = i.fetch_add(1, Ordering::Acquire);
                    if i == 2 {
                        Ok(())
                    } else {
                        Err(MyCustomError::OhNo(i))
                    }
                }
            })
            .await?;

        let i = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let result = executor
            .spawn([Duration::from_millis(0)], || {
                let i = i.clone();
                async move {
                    let i = i.fetch_add(1, Ordering::Acquire);
                    if i == 2 {
                        Ok(())
                    } else {
                        Err(MyCustomError::OhNo(i))
                    }
                }
            })
            .await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_circuit_open() -> Result<(), Box<dyn std::error::Error>> {
        let circuit_breaker = ExecutorConfig::new(Duration::from_secs(3))
            .threshold(0.5)
            .closed_len(NonZeroUsize::new(2).unwrap())
            .open_wait(Duration::from_secs(1));
        let executor = Executor::new(circuit_breaker);

        let first_try = executor
            .spawn([Duration::from_millis(0)], || async move {
                Err::<(), _>(MyCustomError::OhNo(0))
            })
            .await;
        // Circuit is closed, so we get the normal error
        assert!(matches!(
            first_try,
            Err(Error::Inner(MyCustomError::OhNo(0)))
        ));

        let second_try = executor
            .spawn([Duration::from_millis(0); 1], || async move {
                Err::<(), _>(MyCustomError::OhNo(1))
            })
            .await;
        // Circuit has opened, immediate rejection
        assert!(matches!(second_try, Err(Error::Rejected)));

        // Sleep until we can start calculating a new failure rate
        tokio::time::sleep(Duration::from_millis(1_500)).await;

        let third_try = executor
            .spawn([Duration::from_millis(0)], || async move {
                Ok::<(), MyCustomError>(())
            })
            .await;
        // Circuit is closed
        assert!(matches!(third_try, Ok(())));
        Ok(())
    }
}
