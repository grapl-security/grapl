use rusoto_s3::{
    GetObjectRequest as InnerGetObjectRequest,
};
use tokio::io::AsyncReadExt;
use crate::s3::bucket::Bucket;
use crate::s3::client::S3Common;
use crate::TmpError;

#[async_trait::async_trait]
impl<S> ReadS3<S> for Bucket<S>
    where S: S3Common
{
    async fn get_object(&self, key: String) -> GetObjectRequest<S> {
        GetObjectRequest {
            bucket: self.clone(),
            key,
        }
    }
}


#[async_trait::async_trait]
pub trait ReadS3<S: S3Common> {
    async fn get_object(&self, key: String) -> GetObjectRequest<S>;
}



#[derive(Clone)]
pub struct GetObjectRequest<S>
    where S: S3Common
{
    bucket: Bucket<S>,
    key: String,
}

impl<S> GetObjectRequest<S>
    where S: S3Common
{
    pub async fn send(self) -> Result<Vec<u8>, TmpError> {
        let get_object_output = self
            .bucket.s3_client.s3_client
            .get_object(InnerGetObjectRequest {
                bucket: self.bucket.bucket_name.clone(),
                key: self.key,
                expected_bucket_owner: Some(self.bucket.bucket_account_id.clone()),
                ..Default::default()
            })
            .await?;

        let stream = get_object_output.body.expect("todo");

        let mut plugin_binary = Vec::new();

        // read the whole file
        stream
            .into_async_read()
            .read_to_end(&mut plugin_binary)
            .await?;
        Ok(plugin_binary)
    }
}

