pub mod action;
use action::Action;

use std::future::Future;
use std::iter::{IntoIterator, Iterator};
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use tokio::time::{sleep_until, Duration, Instant, Sleep};
use recloser::{AsyncRecloser, AnyError, RecloserFuture, Recloser, RecloserBuilder};

pub use tokio_retry::strategy;

#[derive(thiserror::Error, Debug)]
pub enum Error<E: std::error::Error> {
    #[error("Reject")]
    Rejected,
    #[error(transparent)]
    Inner(E)
}

#[pin_project(project = ExecuteStateProj)]
enum ExecuteState<A>
    where
        A: Action,
{
    Running(#[pin] RecloserFuture<<A as Action>::Future, AnyError>),
    Sleeping(#[pin] Sleep),
}

impl<A: Action> ExecuteState<A> {
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> ExecuteFuturePoll<A> {
        match self.project() {
            ExecuteStateProj::Running(future) => {
                match future.poll(cx) {
                    Poll::Ready(Err(recloser::Error::Inner(e))) => ExecuteFuturePoll::Running(Poll::Ready(Err(Error::Inner(e)))),
                    Poll::Ready(Err(recloser::Error::Rejected)) => ExecuteFuturePoll::Running(Poll::Ready(Err(Error::Rejected))),
                    Poll::Ready(Ok(p)) => ExecuteFuturePoll::Running(Poll::Ready(Ok(p))),
                    Poll::Pending => ExecuteFuturePoll::Running(Poll::Pending),
                }
            },
            ExecuteStateProj::Sleeping(future) => ExecuteFuturePoll::Sleeping(future.poll(cx)),
        }
    }
}

enum ExecuteFuturePoll<A>
    where
        A: Action,
{
    Running(Poll<Result<<A as Action>::Item, Error<<A as Action>::Error>>>),
    Sleeping(Poll<()>),
}

pub type ActionResult<A> = Result<<A as Action>::Item, Error<<A as Action>::Error>>;

/// Future that drives multiple attempts at an action via a retry strategy. Retries are only attempted if
/// the `Error` returned by the future satisfies a given condition.
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
    action: A,
}

impl<'a, I, A> Execution<'a, I, A>
    where
        I: Iterator<Item = Duration>,
        A: Action,
{
    pub fn spawn<T: IntoIterator<IntoIter = I, Item = Duration>>(
        strategy: T,
        recloser: &'a AsyncRecloser,
        mut action: A,
    ) -> Execution<'a, I, A> {
        Execution {
            strategy: strategy.into_iter(),
            state: ExecuteState::Running(recloser.call(action.run())),
            recloser,
            action,
        }
    }

    fn attempt(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ActionResult<A>> {
        // check circuit breaker
        let future = {
            let this = self.as_mut().project();
            this.recloser.call(this.action.run())
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
            },
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
                Poll::Ready(Err(err)) => {
                    match self.retry(err, cx) {
                        Ok(poll) => poll,
                        Err(err) => Poll::Ready(Err(err)),
                    }
                }
            },
            ExecuteFuturePoll::Sleeping(poll_result) => match poll_result {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => self.attempt(cx),
            },
        }
    }
}

pub struct CircuitBreakerConfig {
    builder: RecloserBuilder,
}

impl CircuitBreakerConfig {
    pub fn error_rate(mut self, threshold: f32) -> Self {
        self.builder = self.builder.error_rate(threshold);
        self
    }

    pub fn closed_len(mut self, closed_len: usize) -> Self {
        self.builder = self.builder.closed_len(closed_len);
        self
    }

    pub fn half_open_len(mut self, half_open_len: usize) -> Self {
        self.builder = self.builder.half_open_len(half_open_len);
        self
    }

    pub fn open_wait(mut self, open_wait: Duration) -> Self {
        self.builder = self.builder.open_wait(open_wait);
        self
    }

    fn build(self) -> Recloser {
        self.builder.build()
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        CircuitBreakerConfig {
            builder: Recloser::custom(),
        }
    }
}

pub struct Executor
{
    recloser: AsyncRecloser,
}

impl Executor

{
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Executor {
            recloser: AsyncRecloser::from(config.build()),
        }
    }

    pub fn spawn<A, I, T>(
        &self,
        strategy: T,
        mut action: A,
    ) -> Execution<I, A>
        where
            I: Iterator<Item = Duration>,
            A: Action,
            T: IntoIterator<IntoIter = I, Item = Duration>
    {
        Execution {
            strategy: strategy.into_iter(),
            state: ExecuteState::Running(self.recloser.call(action.run())),
            recloser: &self.recloser,
            action,
        }
    }
}



#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use super::*;

    #[derive(thiserror::Error, Debug)]
    enum MyCustomError {
        #[error("OhNo!")]
        OhNo(i32)
    }

    #[tokio::test]
    async fn test_retries() -> Result<(), Box<dyn std::error::Error>> {
        let executor = Executor::new(CircuitBreakerConfig::default());

        executor.spawn(
            [],
            || {
                async move {
                    Ok::<(), MyCustomError>(())
                }
            },
        ).await?;

        let i = Arc::new(std::sync::atomic::AtomicI32::new(0));
        executor.spawn(
            [Duration::from_millis(0), Duration::from_millis(0)],
            || {
                let i = i.clone();
                async move {
                    let i = i.fetch_add(1, Ordering::Acquire);
                    if i == 2 {
                        Ok(())
                    } else {
                        Err(MyCustomError::OhNo(i))
                    }
                }
            },
        ).await?;

        let i = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let result = executor.spawn(
            [Duration::from_millis(0)],
            || {
                let i = i.clone();
                async move {
                    let i = i.fetch_add(1, Ordering::Acquire);
                    if i == 2 {
                        Ok(())
                    } else {
                        Err(MyCustomError::OhNo(i))
                    }
                }
            },
        ).await;
        assert!(result.is_err());
        
        Ok(())
    }
}