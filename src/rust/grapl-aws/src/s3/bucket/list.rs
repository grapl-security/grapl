#![allow(warnings)]

use rusoto_s3::{ListObjectsV2Request as InnerListObjectsV2Request, ListObjectsV2Output, S3};
use async_stream::try_stream;
use futures_core::stream::Stream;

use tokio::io::AsyncReadExt;
use crate::s3::bucket::Bucket;
use crate::s3::client::S3Common;
use crate::TmpError;
use rusoto_s3::Object;

#[async_trait::async_trait]
pub trait ListS3<S: S3Common> {
    async fn list_objects(&self) -> ListObjectRequest<S>;
}


#[derive(Clone)]
pub struct ListObjectRequest<S>
    where S: S3Common
{
    bucket: Bucket<S>,
    prefix: Option<String>,
}

impl<S> ListObjectRequest<S>
    where S: S3Common
{
    pub async fn send(&self) -> impl Stream<Item=Result<Object, TmpError>> + '_ {
        try_stream! {
            let resolution = self.send_once(None).await?;
            let contents = match resolution.contents {
                Some(contents) => contents,
                None => return
            };

            for object in contents {
                yield object
            }

            let mut continuation_token: Option<String> = resolution.continuation_token.clone();
            while let Some(token) = continuation_token {
                tokio::task::yield_now().await;
                let resolution = self.send_once(Some(token)).await?;
                let contents = match resolution.contents {
                    Some(contents) => contents,
                    None => return
                };
                continuation_token = resolution.continuation_token.clone();

                for object in contents {
                    yield object
                }
            }
        }
    }

    async fn send_once(&self, continuation_token: Option<String>) -> Result<ListObjectsV2Output, TmpError> {
        let response = self.bucket.s3_client.s3_client.list_objects_v2(
            InnerListObjectsV2Request {
                bucket: self.bucket.bucket_name.clone(),
                continuation_token,
                delimiter: None,
                encoding_type: None,
                expected_bucket_owner: Some(self.bucket.bucket_account_id.clone()),
                fetch_owner: Some(false),
                max_keys: None,
                prefix: self.prefix.clone(),
                request_payer: None,
                start_after: None
            }
        ).await?;

        Ok(response)
    }
}