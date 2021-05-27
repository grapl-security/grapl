use std::io::Stdout;

use async_trait::async_trait;
use grapl_observe::{
    metric_reporter::{
        tag,
        MetricReporter,
    },
    timers::TimedFutureExt,
};
use rusoto_core::RusotoError;
use rusoto_s3::{
    PutObjectError,
    PutObjectRequest,
    S3,
};
use rusoto_sqs::SendMessageError;
use tokio::time::{
    error::Elapsed,
    Duration,
};
use tracing::error;

use crate::{
    errors::{
        CheckedError,
        Recoverable,
    },
    event_emitter::EventEmitter,
};

#[derive(thiserror::Error, Debug)]
pub enum S3EventEmitterError {
    #[error("PutObjectError: {0}")]
    PutObjectError(#[from] RusotoError<PutObjectError>),
    #[error("Timeout")]
    Timeout(#[from] Elapsed),
}

pub struct S3EventEmitter<S, F>
where
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
{
    s3: S,
    output_bucket: String,
    key_fn: F,
    metric_reporter: MetricReporter<Stdout>,
}

impl<S, F> Clone for S3EventEmitter<S, F>
where
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
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
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
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
impl<S, F> EventEmitter for S3EventEmitter<S, F>
where
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
{
    type Event = Vec<u8>;
    type Error = S3EventEmitterError;

    #[tracing::instrument(skip(self, events))]
    async fn emit_event(&mut self, events: Vec<Self::Event>) -> Result<(), Self::Error> {
        let mut event_uploads = Vec::with_capacity(events.len());

        for event in events {
            let output_bucket = self.output_bucket.clone();
            let key = (self.key_fn)(&event);
            let s3 = self.s3.clone();
            let mut metric_reporter = self.metric_reporter.clone();
            let put_object = async move {
                tracing::info!("uploading event to: {} {}", output_bucket, key);
                let (res, ms) = s3
                    .put_object(PutObjectRequest {
                        body: Some(event.into()),
                        bucket: output_bucket.clone(),
                        key: key.clone(),
                        ..Default::default()
                    })
                    .timed()
                    .await;
                metric_reporter
                    .histogram(
                        "s3_event_emitter.emit_event.ms",
                        ms as f64,
                        &[tag("success", res.is_ok())],
                    )
                    .unwrap_or_else(|e| error!("s3_event_emitter.emit_event.ms: {:?}", e));

                match res {
                    Ok(res) => {
                        tracing::debug!("PutObject succeeded: {:?}", res);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::warn!("PutObject failed: {:?}", e);
                        Err(S3EventEmitterError::from(e))
                    }
                }
            };
            event_uploads.push(put_object);
        }
        let event_uploads = tokio::time::timeout(
            Duration::from_secs(3),
            futures::future::join_all(event_uploads),
        )
        .await?;

        for e in event_uploads.into_iter() {
            e?;
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum S3NotificationError {
    #[error("PublishError")]
    PublishError(#[from] RusotoError<SendMessageError>),
}

impl CheckedError for S3NotificationError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::PublishError(e) => e.error_type(),
        }
    }
}
