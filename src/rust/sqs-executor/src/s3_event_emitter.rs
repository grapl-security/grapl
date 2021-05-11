use std::io::Stdout;

use async_trait::async_trait;
use aws_lambda_events::event::s3::{
    S3Bucket,
    S3Entity,
    S3Event,
    S3EventRecord,
    S3Object,
    S3RequestParameters,
    S3UserIdentity,
};
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
use rusoto_sqs::{
    SendMessageError,
    SendMessageRequest,
    Sqs,
    SqsClient,
};
use tap::prelude::TapFallible;
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
pub enum S3EventEmitterError<OnEmitError>
where
    OnEmitError: CheckedError + Send,
{
    #[error("PutObjectError: {0}")]
    PutObjectError(#[from] RusotoError<PutObjectError>),
    #[error("Timeout")]
    Timeout(#[from] Elapsed),
    #[error("OnEventEmit")]
    OnEmitError(OnEmitError),
}

impl<OnEmitError> CheckedError for S3EventEmitterError<OnEmitError>
where
    OnEmitError: CheckedError + Send,
{
    fn error_type(&self) -> Recoverable {
        match self {
            Self::PutObjectError(e) => e.error_type(),
            Self::Timeout(_) => Recoverable::Transient,
            Self::OnEmitError(e) => e.error_type(),
        }
    }
}

pub struct S3EventEmitter<S, F, OnEmit, OnEmitError>
where
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
    OnEmit: Clone + OnEventEmit<Error = OnEmitError> + Send + Sync + 'static,
    OnEmitError: CheckedError + Send,
{
    s3: S,
    output_bucket: String,
    key_fn: F,
    metric_reporter: MetricReporter<Stdout>,
    on_emit: OnEmit,
    _p: std::marker::PhantomData<OnEmit>,
}

impl<S, F, OnEmit, OnEmitError> Clone for S3EventEmitter<S, F, OnEmit, OnEmitError>
where
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
    OnEmit: Clone + OnEventEmit<Error = OnEmitError> + Send + Sync + 'static,
    OnEmitError: CheckedError + Send,
{
    fn clone(&self) -> Self {
        Self {
            s3: self.s3.clone(),
            output_bucket: self.output_bucket.clone(),
            key_fn: self.key_fn.clone(),
            metric_reporter: self.metric_reporter.clone(),
            on_emit: self.on_emit.clone(),
            _p: std::marker::PhantomData,
        }
    }
}

impl<S, F, OnEmit, OnEmitError> S3EventEmitter<S, F, OnEmit, OnEmitError>
where
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
    OnEmit: Clone + OnEventEmit<Error = OnEmitError> + Send + Sync + 'static,
    OnEmitError: CheckedError + Send,
{
    pub fn new(
        s3: S,
        output_bucket: impl Into<String>,
        key_fn: F,
        on_emit: OnEmit,
        metric_reporter: MetricReporter<Stdout>,
    ) -> Self {
        let output_bucket = output_bucket.into();
        Self {
            s3,
            output_bucket,
            key_fn,
            metric_reporter,
            on_emit,
            _p: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, F, OnEmit, OnEmitError> EventEmitter for S3EventEmitter<S, F, OnEmit, OnEmitError>
where
    S: Clone + S3 + Send + Sync,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
    OnEmit: Clone + OnEventEmit<Error = OnEmitError> + Send + Sync + 'static,
    OnEmitError: CheckedError + Send,
{
    type Event = Vec<u8>;
    type Error = S3EventEmitterError<OnEmitError>;

    #[tracing::instrument(skip(self, events))]
    async fn emit_event(&mut self, events: Vec<Self::Event>) -> Result<(), Self::Error> {
        let mut event_uploads = Vec::with_capacity(events.len());

        for event in events {
            let on_emit = self.on_emit.clone();
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
                        on_emit
                            .event_notification(output_bucket, key)
                            .await
                            .map_err(S3EventEmitterError::OnEmitError)
                            .tap_err(|e| error!("event_notification failed: {:?}", e))?;
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

#[async_trait]
pub trait OnEventEmit {
    type Error: CheckedError;
    async fn event_notification(&self, bucket: String, key: String) -> Result<(), Self::Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum NopEventEmitterError {
    #[error("Never")]
    Never,
}

pub struct S3ToSqsEventNotifier<S>
where
    S: Sqs + Clone + Send + Sync + 'static,
{
    enabled: bool,
    sqs_client: S,
    dest_queue_url: String,
}

impl<S> Clone for S3ToSqsEventNotifier<S>
where
    S: Sqs + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            sqs_client: self.sqs_client.clone(),
            dest_queue_url: self.dest_queue_url.clone(),
        }
    }
}

impl<S> S3ToSqsEventNotifier<S>
where
    S: Sqs + Clone + Send + Sync + 'static,
{
    pub fn new(enabled: bool, sqs_client: S, dest_queue_url: String) -> Self {
        Self {
            enabled,
            sqs_client,
            dest_queue_url,
        }
    }
}

impl S3ToSqsEventNotifier<SqsClient> {
    pub fn from_sqs_client(enabled: bool, sqs_client: SqsClient, dest_queue_url: String) -> Self {
        Self {
            enabled,
            sqs_client,
            dest_queue_url,
        }
    }
}

#[async_trait]
impl<S> OnEventEmit for S3ToSqsEventNotifier<S>
where
    S: Sqs + Clone + Send + Sync + 'static,
{
    type Error = S3NotificationError;
    async fn event_notification(&self, bucket: String, key: String) -> Result<(), Self::Error> {
        if !self.enabled {
            tracing::debug!("event notifications are disabled");
            return Ok(());
        }
        tracing::debug!(
            "event_notification: {} {}/{}",
            self.dest_queue_url,
            bucket,
            key
        );

        send_s3_notification(
            self.sqs_client.clone(),
            self.dest_queue_url.clone(),
            bucket,
            key,
        )
        .await
    }
}

pub fn s3_event_notification(bucket: String, key: String) -> S3Event {
    S3Event {
        records: vec![S3EventRecord {
            // todo: Take this as an argument
            aws_region: Some("us-east-1".to_owned()),
            event_time: chrono::Utc::now(),
            principal_id: S3UserIdentity { principal_id: None },
            request_parameters: S3RequestParameters {
                source_ip_address: None,
            },
            response_elements: Default::default(),
            s3: S3Entity {
                bucket: S3Bucket {
                    name: Some(bucket),
                    owner_identity: S3UserIdentity { principal_id: None },
                    arn: None,
                },
                object: S3Object {
                    key: Some(key),
                    size: Some(0),
                    url_decoded_key: None,
                    version_id: None,
                    e_tag: None,
                    sequencer: None,
                },
                schema_version: None,
                configuration_id: None,
            },
            event_version: None,
            event_source: None,
            event_name: None,
        }],
    }
}

pub async fn send_s3_notification<S>(
    sqs_client: S,
    dest_queue_url: String,
    bucket: String,
    key: String,
) -> Result<(), S3NotificationError>
where
    S: Clone + Sqs + Send + Sync + 'static,
{
    let output_event = s3_event_notification(bucket, key);
    sqs_client
        .send_message(SendMessageRequest {
            message_body: serde_json::to_string(&output_event).expect("failed to encode s3 event"),
            queue_url: dest_queue_url,
            ..Default::default()
        })
        .await?;
    Ok(())
}
