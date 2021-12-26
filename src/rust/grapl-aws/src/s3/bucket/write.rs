use crate::s3::bucket::Metadata;

use std::collections::HashMap;

use rusoto_s3::PutObjectRequest as InnerPutObjectRequest;

use crate::{
    s3::{
        bucket::Bucket,
        client::S3Common,
    },
    TmpError,
};

pub trait WriteS3<S, const READ_CAP: bool, const LIST_CAP: bool>
where
    S: S3Common,
{
    fn put_object(&self, key: String, object: Vec<u8>) -> PutObjectRequest<S, READ_CAP, LIST_CAP>;
}

impl<S, const READ_CAP: bool, const LIST_CAP: bool> WriteS3<S, READ_CAP, LIST_CAP>
    for Bucket<S, READ_CAP, true, LIST_CAP>
where
    S: S3Common,
{
    fn put_object(&self, key: String, object: Vec<u8>) -> PutObjectRequest<S, READ_CAP, LIST_CAP> {
        PutObjectRequest {
            bucket: self.clone(),
            key,
            object,
            metadata: None,
        }
    }
}

#[derive(Clone)]
pub struct PutObjectRequest<S, const READ_CAP: bool, const LIST_CAP: bool>
where
    S: S3Common,
{
    bucket: Bucket<S, READ_CAP, true, LIST_CAP>,
    key: String,
    object: Vec<u8>,
    metadata: Option<HashMap<String, String>>,
}

impl<S, const READ_CAP: bool, const LIST_CAP: bool> PutObjectRequest<S, READ_CAP, LIST_CAP>
where
    S: S3Common,
{
    pub fn with_metadata(mut self, metadata: impl Metadata) -> Self {
        if let Some(meta) = self.metadata.as_mut() {
            metadata.merge_into(meta);
        }
        self
    }

    pub async fn send(self) -> Result<(), TmpError> {
        self.bucket
            .s3_client
            .s3_client
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
