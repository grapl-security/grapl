extern crate aws_lambda_events;
extern crate zstd;
extern crate base16;
extern crate base64;
extern crate dgraph_rs;
#[macro_use] extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate grpc;
extern crate lambda_runtime as lambda;
#[macro_use] extern crate log;
extern crate openssl_probe;
extern crate prost;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sns;
extern crate rusoto_sqs;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate sha2;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate stopwatch;

use std::str::FromStr;

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use dgraph_rs::protos::api;
use dgraph_rs::protos::api_grpc::{Dgraph, DgraphClient};
use dgraph_rs::protos::api_grpc;
use dgraph_rs::Transaction;
use failure::Error;
use futures::Future;
use graph_descriptions::graph_description::*;
use grpc::{Client, ClientStub};
use grpc::ClientConf;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::lambda;
use prost::Message;
use rusoto_core::Region;
use rusoto_s3::{ListObjectsRequest, PutObjectRequest, S3, S3Client};
use rusoto_sns::{Sns, SnsClient};
use rusoto_sns::PublishInput;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use sha2::{Digest, Sha256};
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_sns_sqs;
use sqs_lambda::NopSqsCompletionHandler;
use sqs_lambda::SnsEventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::EventDecoder;
use std::env;

macro_rules! log_time {
    ($msg:expr, $x:expr) => {
        {
            let mut sw = stopwatch::Stopwatch::start_new();
            #[allow(path_statements)]
            let result = $x;
            sw.stop();
            info!("{} {} milliseconds", $msg, sw.elapsed_ms());
            result
        }
    };
}

#[derive(Debug)]
struct AnalyzerDispatcher<S>
    where S: S3
{
    s3_client: Arc<S>
}

impl<S> Clone for AnalyzerDispatcher<S>
    where S: S3,
{
    fn clone(&self) -> Self {
        Self {
            s3_client: self.s3_client.clone()
        }
    }
}

fn get_s3_keys(s3_client: &impl S3, bucket: impl Into<String>) -> Result<impl IntoIterator<Item=Result<String, Error>>, Error>
{
    let bucket = bucket.into();

    let list_res = s3_client.list_objects(
        &ListObjectsRequest {
            bucket,
            ..Default::default()
        }
    )
        .with_timeout(Duration::from_secs(2))
        .wait()?;

    let contents = match list_res.contents {
        Some(contents) => contents,
        None => {
            warn!("List response returned nothing");
            Vec::new()
        }
    };

    Ok(
        contents.into_iter()
            .map(|object| match object.key {
                Some(key) => Ok(key),
                None => bail!("S3Object is missing key")
            })
    )
}

#[derive(Serialize, Deserialize)]
struct AnalyzerDispatchEvent<'a> {
    #[serde(borrow)]
    key: &'a str,
    #[serde(borrow)]
    subgraph: &'a [u8]
}

fn encode_subgraph(subgraph: &GraphDescription) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::with_capacity(5000);
    subgraph.encode(&mut buf)?;
    Ok(buf)
}

fn emit_dispatch_event(
    s3_client: &impl S3,
    key: impl AsRef<str>,
    subgraph: impl AsRef<[u8]>
) -> Result<(), Error> {

    let event = AnalyzerDispatchEvent {
        key: key.as_ref(),
        subgraph: subgraph.as_ref()
    };


    let body = serde_json::to_vec(&event)?;

    // publish event to s3
    let bucket = std::env::var("DISPATCHED_ANALYZER_BUCKET")
        .expect("DISPATCHED_ANALYZER_TOPIC_ARN");

    let key = {
        let mut hasher = Sha256::default();
        hasher.input(&body);

        let key = hasher.result();
        let key = base16::encode_lower(&key);
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs();

        let day = epoch - (epoch % (24 * 60 * 60));

        format!("{}/{}", day, key)
    };

    info!("Uploading payload to : {}", key);

    s3_client.put_object(&PutObjectRequest {
        bucket,
        key,
        body: Some(body),
        ..Default::default()
    })
        .with_timeout(Duration::from_secs(2))
        .wait()?;


    Ok(())
}

impl<S> EventHandler<GraphDescription> for AnalyzerDispatcher<S>
    where S: S3
{
    fn handle_event(&self, subgraph: GraphDescription) -> Result<(), Error> {
        let bucket = std::env::var("BUCKET_PREFIX")
            .map(|prefix| format!("{}-analyzers-bucket", prefix))
            .expect("BUCKET_PREFIX");

        info!("Retrieving S3 keys");
        let keys = get_s3_keys(self.s3_client.as_ref(), bucket)?;

        info!("Encoding subgraph");
        let subgraph = encode_subgraph(&subgraph)?;

        info!("Creating sns_client");
        let region = {
            let region_str = env::var("AWS_REGION").expect("AWS_REGION");
            Region::from_str(&region_str).expect("Invalid Region")
        };
        let s3_client = S3Client::simple(region);

        keys.into_iter()
            .map(|key| emit_dispatch_event(&s3_client,key?, &subgraph))
            .collect()
    }
}


#[derive(Debug, Clone)]
pub struct Base64ZstdProtoDecoder;

impl<E> EventDecoder<E> for Base64ZstdProtoDecoder
    where E: Message + Default
{

    fn decode(&mut self, body: Vec<u8>) -> Result<E, Error>
        where E: Message + Default,
    {
        let decoded = base64::decode(&body)?;

        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&decoded);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(E::decode(decompressed)?)
    }
}


pub fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let region = {
        let region_str = env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Invalid Region")
    };

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::simple(region.clone()));
    let handler = AnalyzerDispatcher{s3_client};

    info!("Creating retriever");

    let retriever = SnsEventRetriever::new(
        |d| {info!("Parsing: {:?}", d); events_from_sns_sqs(d)},
        Base64ZstdProtoDecoder{},
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = NopSqsCompletionHandler::new(
        queue_url
    );

    let mut sqs_service = SqsService::new(
        retriever,
        handler,
        sqs_completion_handler,
    );

    info!("Handing off event");
    log_time!("sqs_service.run", sqs_service.run(event, ctx)?);

    Ok(())
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    openssl_probe::init_ssl_cert_env_vars();

    lambda!(handler);
}

