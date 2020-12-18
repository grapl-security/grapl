use std::error::Error;

use crate::event_emitter::EventEmitter;
use async_trait::async_trait;

use rusoto_s3::{PutObjectRequest, S3, PutObjectError};
use std::future::Future;
use rusoto_core::RusotoError;

#[derive(Clone)]
pub struct S3EventEmitter<S, F, OnEmission, EmissionResult>
where
    S: S3 + Send + 'static,
    F: Fn(&[u8]) -> String,
    EmissionResult:
        Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
    OnEmission: Fn(String, String) -> EmissionResult + Send + Sync + 'static,
{
    s3: S,
    output_bucket: String,
    key_fn: F,
    on_emission: OnEmission,
}

impl<S, F, OnEmission, EmissionResult> S3EventEmitter<S, F, OnEmission, EmissionResult>
where
    S: S3 + Send + 'static,
    F: Fn(&[u8]) -> String,
    EmissionResult:
        Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
    OnEmission: Fn(String, String) -> EmissionResult + Send + Sync + 'static,
{
    pub fn new(
        s3: S,
        output_bucket: impl Into<String>,
        key_fn: F,
        on_emission: OnEmission,
    ) -> Self {
        let output_bucket = output_bucket.into();
        Self {
            s3,
            output_bucket,
            key_fn,
            on_emission,
        }
    }
}

#[async_trait]
impl<S, F, OnEmission, EmissionResult> EventEmitter
    for S3EventEmitter<S, F, OnEmission, EmissionResult>
where
    S: S3 + Send + Sync + 'static,
    F: Fn(&[u8]) -> String + Send + Sync,
    EmissionResult:
        Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
    OnEmission: Fn(String, String) -> EmissionResult + Send + Sync + 'static,
{
    type Event = Vec<u8>;
    type Error = RusotoError<PutObjectError>;

    #[tracing::instrument(skip(self, events))]
    async fn emit_event(&mut self, events: Vec<Self::Event>) -> Result<(), Self::Error> {
        for event in events {
            let key = (self.key_fn)(&event);
            self.s3
                .put_object(PutObjectRequest {
                    body: Some(event.into()),
                    bucket: self.output_bucket.clone(),
                    key: key.clone(),
                    ..Default::default()
                })
                .await?;

            // TODO: We shouldn't panic when this happens, we should retry or move on to the next event
            (self.on_emission)(self.output_bucket.clone(), key.clone())
                .await
                .expect("on_emission failed");
        }

        // let event_uploads = tokio::time::timeout(
        //     Duration::from_secs(5),
        //     futures::future::join_all(event_uploads)
        // ).await?;

        // let mut err = None;
        // for upload in event_uploads {
        //     // upload?;
        // }

        Ok(())
    }
}
