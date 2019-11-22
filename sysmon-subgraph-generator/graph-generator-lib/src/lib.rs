extern crate base16;
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
#[macro_use] extern crate log;
extern crate prost;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate serde_json;
extern crate sha2;
extern crate zstd;

use std::io::Cursor;
use std::str::FromStr;
use std::time::{SystemTime, Duration};
use std::time::UNIX_EPOCH;

use failure::Error;
use futures::future::Future;
use graph_descriptions::graph_description::*;
use prost::Message;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, S3, S3Client};
use serde_json::Value;
use sha2::{Digest, Sha256};

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
    let region = {
        let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Invalid Region")
    };
    let s3_client = S3Client::new(region);

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    s3_client.put_object(PutObjectRequest {
        bucket: bucket_prefix + "-grapl-raw-log-bucket",
        key,
        body: Some(logs.into()),
        ..Default::default()
    }).sync()?;
    info!("Uploaded raw-logs");
    Ok(())
}

pub fn upload_subgraphs<S>(s3_client: &S, subgraphs: GeneratedSubgraphs) -> Result<(), Error>
    where S: S3
{
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

    // Key is day/time-hash
    let key = format!("{}/{}-{}",
                      day,
                      epoch,
                      base16::encode_lower(&key)
    );

    let mut compressed = Vec::with_capacity(proto.len());
    let mut proto = Cursor::new(&proto);
    zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
        .expect("compress zstd capnp");

    info!("uploading unidentifed_subgraphs to {}", key);

    s3_client.put_object(PutObjectRequest {
        bucket,
        key,
        body: Some(compressed.into()),
        ..Default::default()
    }).with_timeout(Duration::from_secs(5)).sync()?;

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





