use std::error::Error;
use std::io::Stdout;
use std::marker::PhantomData;
use std::time::Duration;

use rusoto_s3::{GetObjectRequest, S3};
use rusoto_sqs::Message as SqsMessage;
use tracing::{debug, error, info};

use async_trait::async_trait;

use crate::event_decoder::PayloadDecoder;
use grapl_observe::metric_reporter::MetricReporter;
use grapl_observe::timers::time_fut_ms;
use std::collections::HashMap;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait PayloadRetriever<T> {
    type Message;
    async fn retrieve_event(&mut self, msg: &Self::Message) -> Result<Option<T>, Box<dyn Error>>;
}

#[derive(Clone)]
pub struct S3PayloadRetriever<S, SInit, D, E>
where
    S: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S) + Clone + Send + Sync + 'static,
    D: PayloadDecoder<E> + Clone + Send + 'static,
    E: Send + 'static,
{
    s3_init: SInit,
    s3_clients: HashMap<String, S>,
    decoder: D,
    metric_reporter: MetricReporter<Stdout>,
    phantom: PhantomData<E>,
}

impl<S, SInit, D, E> S3PayloadRetriever<S, SInit, D, E>
where
    S: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S) + Clone + Send + Sync + 'static,
    D: PayloadDecoder<E> + Clone + Send + 'static,
    E: Send + 'static,
{
    pub fn new(s3: SInit, decoder: D, metric_reporter: MetricReporter<Stdout>) -> Self {
        Self {
            s3_init: s3,
            s3_clients: HashMap::new(),
            decoder,
            metric_reporter,
            phantom: PhantomData,
        }
    }

    pub fn get_client(&mut self, region: String) -> S {
        match self.s3_clients.get(&region) {
            Some(s3) => s3.clone(),
            None => {
                let client = (self.s3_init)(region.clone());
                self.s3_clients.insert(region.to_string(), client.clone());
                client
            }
        }
    }
}

#[async_trait]
impl<S, SInit, D, E> PayloadRetriever<E> for S3PayloadRetriever<S, SInit, D, E>
where
    S: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S) + Clone + Send + Sync + 'static,
    D: PayloadDecoder<E> + Clone + Send + 'static,
    E: Send + 'static,
{
    type Message = SqsMessage;
    #[tracing::instrument(skip(self, msg))]
    async fn retrieve_event(&mut self, msg: &Self::Message) -> Result<Option<E>, Box<dyn Error>> {
        let body = msg.body.as_ref().unwrap();
        debug!("Got body from message: {}", body);
        let event: serde_json::Value = serde_json::from_str(body)?;

        if let Some(Some(event_str)) = event.get("Event").map(serde_json::Value::as_str) {
            if event_str == "s3:TestEvent" {
                return Ok(None);
            }
        }
        let record = &event["Records"][0]["s3"];

        let bucket = record["bucket"]["name"].as_str().expect("bucket name");
        let key = record["object"]["key"].as_str().expect("object key");
        debug!("Retrieving S3 payload from: {} / {}", bucket, key);
        let region = &event["Records"][0]["awsRegion"].as_str().expect("region");
        let s3 = self.get_client(region.to_string());
        let s3_data = s3.get_object(GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        });

        let s3_data = tokio::time::timeout(Duration::from_secs(5), s3_data);
        let (s3_data, ms) = time_fut_ms(s3_data).await;
        let s3_data = s3_data??;
        self.metric_reporter
            .histogram("s3_consumer.get_object.ms", ms as f64, &[])
            .unwrap_or_else(|e| error!("failed to report s3_consumer.get_object.ms: {:?}", e));

        let object_size = record["object"]["size"].as_u64().unwrap_or_default();
        let prealloc = if object_size < 1024 {
            1024
        } else {
            object_size as usize
        };

        info!("Retrieved s3 payload with size : {:?}", prealloc);

        let mut body = Vec::with_capacity(prealloc);

        s3_data
            .body
            .expect("Missing S3 body")
            .into_async_read()
            .read_to_end(&mut body)
            .await?;

        self.metric_reporter
            .gauge("s3_retriever.bytes", body.len() as f64, &[])
            .unwrap_or_else(|e| error!("failed to report s3_retriever.bytes: {:?}", e));

        info!("Read s3 payload body");
        self.decoder.decode(body).map(Option::from)
    }
}
