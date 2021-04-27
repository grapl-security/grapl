use std::{future::Future,
          io::Stdout,
          pin::Pin,
          task::{Context,
                 Poll},
          time::Instant};

use pin_project::pin_project;

use crate::metric_reporter::{MetricReporter,
                             TagPair};

pub fn time_it<F, R>(f: F) -> (R, std::time::Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    (f(), start.elapsed())
}

pub fn time_it_ms<F, R>(f: F) -> (R, u64)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let res = f();
    (res, start.elapsed().as_millis() as u64)
}

pub async fn time_fut<F, R>(f: F) -> (R, std::time::Duration)
where
    F: Future<Output = R>,
{
    let start = Instant::now();
    (f.await, start.elapsed())
}

pub async fn time_fut_ms<F, R>(f: F) -> (R, u64)
where
    F: Future<Output = R>,
{
    let start = Instant::now();
    let res = f.await;
    (res, start.elapsed().as_millis() as u64)
}

impl<T> TimedFutureExt for T where T: Future {}

pub trait TimedFutureExt: Future {
    fn timed(self) -> Timed<Self>
    where
        Self: Sized,
    {
        Timed::new(self)
    }
}

#[pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Timed<Fut>(#[pin] Fut, Option<Instant>);

impl<Fut> Timed<Fut>
where
    Fut: Future,
{
    pub(super) fn new(future: Fut) -> Timed<Fut> {
        Timed(future, None)
    }
}

impl<Fut> Future for Timed<Fut>
where
    Fut: Future,
{
    type Output = (Fut::Output, u64);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut _self = self.project();

        if _self.1.is_none() {
            *_self.1 = Some(Instant::now());
        }
        match _self.0.poll(cx) {
            Poll::Ready(result) => {
                Poll::Ready((result, _self.1.unwrap().elapsed().as_millis() as u64))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

pub trait HistogramFutExt<'a>: Future + 'a {
    fn histogram(
        self,
        msg: impl Into<String>,
        tags: &'a [TagPair<'a>],
        m: &'a mut MetricReporter<Stdout>,
    ) -> HistoGramFut<'a, Self>
    where
        Self: Sized,
    {
        HistoGramFut::new(self, msg, tags, m)
    }
}

#[pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct HistoGramFut<'a, Fut>(
    #[pin] Fut,
    Option<Instant>,
    String,
    &'a [TagPair<'a>],
    &'a mut MetricReporter<Stdout>,
);

impl<'a, T> HistogramFutExt<'a> for T where T: Future + 'a {}

impl<'a, Fut> HistoGramFut<'a, Fut>
where
    Fut: Future,
{
    pub(super) fn new(
        future: Fut,
        msg: impl Into<String>,
        tags: &'a [TagPair<'a>],
        m: &'a mut MetricReporter<Stdout>,
    ) -> Self {
        HistoGramFut(future, None, msg.into(), tags, m)
    }
}

impl<'a, Fut> Future for HistoGramFut<'a, Fut>
where
    Fut: Future,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut _self = self.project();

        if _self.1.is_none() {
            *_self.1 = Some(Instant::now());
        }
        match _self.0.poll(cx) {
            Poll::Ready(result) => {
                let ms = _self.1.unwrap().elapsed().as_millis() as f64;
                let msg = _self.2;
                let tags = _self.3;
                let metric_reporter: &mut &mut MetricReporter<_> = _self.4;
                let _ = metric_reporter.histogram(msg, ms, tags);
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
