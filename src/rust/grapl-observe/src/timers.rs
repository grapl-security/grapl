use std::future::Future;
use stopwatch::Stopwatch;

pub fn time_it<F, R>(f: F) -> (R, std::time::Duration)
where
    F: FnOnce() -> R,
{
    let sw = Stopwatch::start_new();
    (f(), sw.elapsed())
}

pub fn time_it_ms<F, R>(f: F) -> (R, u64)
where
    F: FnOnce() -> R,
{
    let sw = Stopwatch::start_new();
    let res = f();
    (res, sw.elapsed_ms() as u64)
}

pub async fn time_fut<F, R>(f: F) -> (R, std::time::Duration)
where
    F: Future<Output = R>,
{
    let sw = Stopwatch::start_new();
    (f.await, sw.elapsed())
}

pub async fn time_fut_ms<F, R>(f: F) -> (R, u64)
where
    F: Future<Output = R>,
{
    let sw = Stopwatch::start_new();
    let res = f.await;
    (res, sw.elapsed_ms() as u64)
}

use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<T> TimedFutureExt for T where T: Future {}

pub trait TimedFutureExt: Future {
    fn timed(self) -> Timed<Self>
    where
        Self: Sized,
    {
        Timed::new(self)
    }
}
/// Future for the [`catch_unwind`](super::FutureExt::catch_unwind) method.
#[pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Timed<Fut>(#[pin] Fut, Stopwatch);

impl<Fut> Timed<Fut>
where
    Fut: Future,
{
    pub(super) fn new(future: Fut) -> Timed<Fut> {
        Timed(future, Stopwatch::new())
    }
}

impl<Fut> Future for Timed<Fut>
where
    Fut: Future,
{
    type Output = (Fut::Output, u64);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut _self = self.project();

        if !_self.1.is_running() {
            _self.1.start();
        }
        match _self.0.poll(cx) {
            Poll::Ready(result) => Poll::Ready((result, _self.1.elapsed_ms() as u64)),
            Poll::Pending => Poll::Pending,
        }
        // catch_unwind(AssertUnwindSafe(|| ))?.map(Ok)
        // (ready, 0)
    }
}
