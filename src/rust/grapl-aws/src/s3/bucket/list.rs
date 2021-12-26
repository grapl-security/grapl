use rusoto_s3::{ListObjectsV2Request as InnerListObjectsV2Request, ListObjectsV2Output};
use async_stream::try_stream;
use futures_core::stream::Stream;

use crate::s3::bucket::Bucket;
use crate::s3::client::S3Common;
use crate::TmpError;
use rusoto_s3::Object;

pub trait ListS3<S, const READ_CAP: bool, const WRITE_CAP: bool>
    where S: S3Common
{
    fn list_objects(&self) -> ListObjectRequest<S, READ_CAP, WRITE_CAP,>;
}

impl<S, const READ_CAP: bool, const WRITE_CAP: bool> ListS3<S, READ_CAP, WRITE_CAP> for Bucket<S, READ_CAP, WRITE_CAP, true>
    where S: S3Common
{
    fn list_objects(&self) -> ListObjectRequest<S, READ_CAP, WRITE_CAP> {
        ListObjectRequest {
            bucket: self.clone(),
            prefix: None,
        }
    }
}


#[derive(Clone)]
pub struct ListObjectRequest<S, const READ_CAP: bool, const WRITE_CAP: bool>
    where S: S3Common
{
    bucket: Bucket<S, READ_CAP, WRITE_CAP, true>,
    prefix: Option<String>,
}

impl<S, const READ_CAP: bool, const WRITE_CAP: bool> ListObjectRequest<S, READ_CAP, WRITE_CAP>
    where S: S3Common
{
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }

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