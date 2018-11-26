#[macro_use] extern crate log;

extern crate aws_lambda as lambda;
extern crate failure;
extern crate futures;
extern crate base16;
extern crate prost;
extern crate sha2;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate rusoto_core;
extern crate sqs_microservice;
extern crate graph_descriptions;
extern crate serde_json;
extern crate zstd;

use rusoto_core::Region;
use rusoto_s3::{S3, S3Client, PutObjectRequest};
use failure::Error;
use prost::Message;

use serde_json::Value;

use futures::future::Future;

use sha2::{Digest, Sha256};

use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use sqs_microservice::*;

use graph_descriptions::graph_description::*;
use std::io::Cursor;

#[inline(always)]
pub fn handle_json_encoded_logs(f: impl (Fn(Vec<Value>)
    -> Result<Vec<GraphDescription>, Error>) + Clone + Send + 'static)
{
    handle_s3_sns_sqs_json(f, move |subgraphs| {
        info!("Uploading {} subgraphs", subgraphs.len());
        upload_subgraphs(GeneratedSubgraphs::new(subgraphs))
    });
}


pub fn send_logs_to_generators(
    sourcetype: impl AsRef<str>,
    logs: Vec<u8>,
) -> Result<(), Error> {


    let key = {
        let mut hasher = Sha256::default();
        hasher.input(&logs);

        let key = hasher.result();
        let key = base16::encode_lower(&key);
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs();

        let day = epoch - (epoch % (24 * 60 * 60));

        format!("{}/{}/{}", sourcetype.as_ref(), day, key)
    };

    info!("Sending {} logs to {}", logs.len(), key);

    let s3_client = S3Client::simple(Region::UsEast1);

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    s3_client.put_object(&PutObjectRequest {
        bucket: bucket_prefix + "-grapl-raw-log-bucket",
        key,
        body: Some(logs.into()),
        ..Default::default()
    }).wait()?;
    info!("Uploaded raw-logs");
    Ok(())
}

pub fn upload_subgraphs(subgraphs: GeneratedSubgraphs) -> Result<(), Error> {
    // TODO: Preallocate buffers
    info!("upload_subgraphs");
    let mut proto = Vec::with_capacity(5000);
    subgraphs.encode(&mut proto)?;

    let mut hasher = Sha256::default();
    hasher.input(&proto);

    let key = base16::encode_lower(hasher.result().as_ref());

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    let bucket = bucket_prefix + "-unid-subgraphs-generated-bucket";
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Bucket by day
    let day = epoch - (epoch % (24 * 60 * 60));

    let key = format!("{}/{}",
                      day,
                      base16::encode_lower(&key)
    );
    info!("uploading unidentifed_subgraphs to {}", key);

    let s3_client = S3Client::simple(Region::UsEast1);

    let mut compressed = Vec::with_capacity(proto.len());
    let mut proto = Cursor::new(&proto);
    zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
        .expect("compress zstd capnp");

    s3_client.put_object(&PutObjectRequest {
        bucket,
        key,
        body: Some(compressed.into()),
        ..Default::default()
    }).wait()?;

    info!("uploaded unidentified subgraphs");

    Ok(())
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

//rust-musl-builder cargo build --release && cp ./target/x86_64-unknown-linux-musl/release/generic-subgraph-generator . && zip ./generic-subgraph-generator.zip ./generic-subgraph-generator && cp ./generic-subgraph-generator.zip ~/workspace/grapl/grapl-cdk/ && rm ./generic-subgraph-generator.zip




