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
