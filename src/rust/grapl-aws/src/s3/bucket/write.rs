use std::collections::HashMap;
use rusoto_s3::{
    PutObjectRequest as InnerPutObjectRequest,
};
use crate::s3::bucket::Bucket;
use crate::s3::client::S3Common;
use crate::TmpError;

#[async_trait::async_trait]
pub trait WriteS3<S: S3Common>  {
    async fn put_object(&self, key: String, object: Vec<u8>) -> PutObjectRequest<S>;
}


#[async_trait::async_trait]
impl<S> WriteS3<S> for Bucket<S>
    where S: S3Common
{
    async fn put_object(&self, key: String, object: Vec<u8>) -> PutObjectRequest<S> {
        PutObjectRequest {
            bucket: self.clone(),
            key,
            object,
            metadata: None,
        }
    }
}

#[derive(Clone)]
pub struct PutObjectRequest<S>
    where S: S3Common
{
    bucket: Bucket<S>,
    key: String,
    object: Vec<u8>,
    metadata: Option<HashMap<String, String>>,
}

impl<S> PutObjectRequest<S>
    where S: S3Common
{
    pub async fn put_object(self) -> Result<(), TmpError> {
        self.bucket.s3_client.s3_client
            .put_object(InnerPutObjectRequest {
                content_length: Some(self.object.len() as i64),
                body: Some(self.object.into()),
                bucket: self.bucket.bucket_name.clone(),
                key: self.key,
                expected_bucket_owner: Some(self.bucket.bucket_account_id.clone()),
                metadata: self.metadata.clone(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}
