use rusoto_s3::GetObjectRequest as InnerGetObjectRequest;
use tokio::io::AsyncReadExt;

use crate::{
    s3::{
        bucket::Bucket,
        client::S3Common,
    },
    TmpError,
};

impl<S, const WRITE_CAP: bool, const LIST_CAP: bool> ReadS3<S, WRITE_CAP, LIST_CAP>
    for Bucket<S, true, WRITE_CAP, LIST_CAP>
where
    S: S3Common,
{
    fn get_object(&self, key: String) -> GetObjectRequest<S, WRITE_CAP, LIST_CAP> {
        GetObjectRequest {
            bucket: self.clone(),
            key,
        }
    }
}

pub trait ReadS3<S, const WRITE_CAP: bool, const LIST_CAP: bool>
where
    S: S3Common,
{
    fn get_object(&self, key: String) -> GetObjectRequest<S, WRITE_CAP, LIST_CAP>;
}

#[derive(Clone)]
pub struct GetObjectRequest<S, const WRITE_CAP: bool, const LIST_CAP: bool>
where
    S: S3Common,
{
    bucket: Bucket<S, true, WRITE_CAP, LIST_CAP>,
    key: String,
}

impl<S, const WRITE_CAP: bool, const LIST_CAP: bool> GetObjectRequest<S, WRITE_CAP, LIST_CAP>
where
    S: S3Common,
{
    pub async fn send(self) -> Result<Vec<u8>, TmpError> {
        let get_object_output = self
            .bucket
            .s3_client
            .s3_client
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

#[cfg(feature = "integration")]
#[cfg(test)]
mod tests {
    use grapl_config::env_helpers::FromEnv;

    use crate::s3::{
        bucket::{
            Bucket,
            ReadS3,
            WriteS3,
        },
        client::S3Client,
    };

    #[tokio::test]
    async fn test_read_objects() -> Result<(), Box<dyn std::error::Error>> {
        let s3_client = S3Client::new(rusoto_s3::S3Client::from_env());

        let bucket: Bucket<_, true, true, false> = s3_client.bucket(
            "local-test-read-objects-bucket".to_owned(),
            "000000000000".to_owned(),
        );

        bucket
            .put_object("key-0".to_owned(), vec![123])
            .send()
            .await?;

        let object = bucket.get_object("key-0".to_owned()).send().await?;

        assert_eq!(object, vec![123]);

        Ok(())
    }
}
