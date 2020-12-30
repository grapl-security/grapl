use std::error::Error;

use crate::event_emitter::EventEmitter;
use async_trait::async_trait;

use crate::errors::{CheckedError, Recoverable};
use futures_util::FutureExt;
use grapl_observe::metric_reporter::{tag, MetricReporter};
use rusoto_core::RusotoError;
use rusoto_s3::{PutObjectError, PutObjectRequest, S3};
use std::future::Future;
use std::io::Stdout;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum S3EventEmitterError {
    #[error("PutObjectError: {0}")]
    PutObjectError(#[from] RusotoError<PutObjectError>),
    #[error("Timeout: {0}")]
    Timeout(#[from] Elapsed),
}

impl CheckedError for S3EventEmitterError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::PutObjectError(e) => e.error_type(),
            Self::Timeout(_) => Recoverable::Transient,
        }
    }
}

pub struct S3EventEmitter<S, F>
where
    S: S3 + Send + 'static,
    F: Fn(&[u8]) -> String,
{
    s3: S,
    output_bucket: String,
    key_fn: F,
    metric_reporter: MetricReporter<Stdout>,
}

impl<S, F> Clone for S3EventEmitter<S, F>
where
    S: Clone + S3 + Send + 'static,
    F: Clone + Fn(&[u8]) -> String,
{
    fn clone(&self) -> Self {
        Self {
            s3: self.s3.clone(),
            output_bucket: self.output_bucket.clone(),
            key_fn: self.key_fn.clone(),
            metric_reporter: self.metric_reporter.clone(),
        }
    }
}

impl<S, F> S3EventEmitter<S, F>
where
    S: S3 + Send + 'static,
    F: Fn(&[u8]) -> String,
{
    pub fn new(
        s3: S,
        output_bucket: impl Into<String>,
        key_fn: F,
        metric_reporter: MetricReporter<Stdout>,
    ) -> Self {
        let output_bucket = output_bucket.into();
        Self {
            s3,
            output_bucket,
            key_fn,
            metric_reporter,
        }
    }
}

#[async_trait]
impl<S, F> EventEmitter
    for S3EventEmitter<S, F>
where
    S: S3 + Send + Sync + 'static,
    F: Fn(&[u8]) -> String + Send + Sync,
{
    type Event = Vec<u8>;
    type Error = S3EventEmitterError;

    #[tracing::instrument(skip(self, events))]
    async fn emit_event(&mut self, events: Vec<Self::Event>) -> Result<(), Self::Error> {
        let mut event_uploads = Vec::with_capacity(events.len());

        for event in events {
            let output_bucket = self.output_bucket.clone();
            let key = (self.key_fn)(&event);
            let put_object = self
                .s3
                .put_object(PutObjectRequest {
                    body: Some(event.into()),
                    bucket: output_bucket.clone(),
                    key,
                    ..Default::default()
                })
                .timed();
            event_uploads.push(put_object)
        }

        let event_uploads = tokio::time::timeout(
            Duration::from_secs(5),
            futures::future::join_all(event_uploads),
        )
        .await?;

        event_uploads.iter().for_each(move |(res, ms)| {
            self.metric_reporter
                .histogram(
                    "s3_event_emitter.emit_event.ms",
                    *ms as f64,
                    &[tag("success", res.is_ok())],
                )
                .unwrap_or_else(|e| error!("s3_event_emitter.emit_event.ms: {:?}", e));
            if let Err(e) = res {
                error!("Failed to emit event: {:?}", e)
            }
        });
        for (e, _) in event_uploads.into_iter() {
            e?;
        }
        Ok(())
    }
}
use futures_util::TryFutureExt;
use grapl_observe::timers::TimedFutureExt;
use tokio::time::{Duration, Elapsed};
