#[macro_use] extern crate log;

extern crate aws_lambda as lambda;
extern crate failure;
extern crate futures;
extern crate base64;
extern crate prost;
extern crate sha2;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate rusoto_core;
extern crate sqs_microservice;
extern crate graph_descriptions;
extern crate serde_json;

use graph_descriptions::*;

use lambda::event::s3::S3Event;
use lambda::event::sqs::SqsEvent;

use rusoto_core::Region;
use rusoto_s3::{S3, S3Client, PutObjectRequest};
use rusoto_sqs::{Sqs, SqsClient};
use failure::Error;
use prost::Message;

use serde_json::Value;

use futures::future::Future;

use sha2::{Digest, Sha256};

use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::sync::mpsc::channel;
use sqs_microservice::*;


#[inline(always)]
pub fn handle_json_encoded_logs(f: impl (Fn(Vec<Value>)
    -> Result<Vec<GraphDescription>, Error>) + Clone + Send + 'static)
{
    handle_s3_sns_sqs_json(f, move |subgraphs| {
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
        let key = base64::encode(&key);
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs();

        let day = epoch - (epoch % (24 * 60 * 60));

        format!("{}/{}/{}",
                          sourcetype.as_ref(), day, key
        )
    };

    let s3_client =

        S3Client::simple(Region::UsEast1);

    s3_client.put_object(&PutObjectRequest {
        bucket: "raw-logs".into(),
        key,
        body: Some(logs.into()),
        ..Default::default()
    }).wait()?;

    Ok(())
}

pub fn upload_subgraphs(subgraphs: GeneratedSubgraphs) -> Result<(), Error> {
    // TODO: Preallocate buffers
    let mut proto = vec![];
    subgraphs.encode(&mut proto);

    let mut hasher = Sha256::default();
    hasher.input(&proto);

    let key = base64::encode(hasher.result().as_ref());

    let bucket = "grapl-unid-subgraph-generated".to_string();
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Bucket by day
    let day = epoch - (epoch % (24 * 60 * 60));

    let key = format!("{}/{}",
                      day,
                      base64::encode(&key)
    );

    let s3_client = S3Client::simple(Region::UsEast1);

    s3_client.put_object(&PutObjectRequest {
        bucket,
        key,
        body: Some(proto.into()),
        ..Default::default()
    }).wait()?;

    Ok(())
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
