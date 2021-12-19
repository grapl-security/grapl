use rusoto_s3::{
    S3
};
use crate::s3::bucket::{Bucket, ReadS3, WriteS3};


#[derive(Clone)]
pub struct S3Client<S>
    where S: S3Common
{
    #[cfg(not(test))]
    pub(crate) s3_client: S,
    #[cfg(test)]
    pub s3_client: S,
}

impl<S> S3Client<S>
    where S: S3Common
{
    pub(crate) fn unconstrained_bucket(&self, bucket_name: String, bucket_account_id: String) -> Bucket<S> {
        Bucket {
            s3_client: self.clone(),
            bucket_name,
            bucket_account_id,
        }
    }

    pub fn read_only_bucket(
        &self,
        bucket_name: String,
        bucket_account_id: String,
    ) -> impl ReadS3<S> {
        self.unconstrained_bucket(bucket_name, bucket_account_id)
    }

    pub fn write_only_bucket(
        &self,
        bucket_name: String,
        bucket_account_id: String,
    ) -> impl WriteS3<S> {
        self.unconstrained_bucket(bucket_name, bucket_account_id)
    }

    pub fn read_write_bucket(
        &self,
        bucket_name: String,
        bucket_account_id: String,
    ) -> impl ReadS3<S> + WriteS3<S> {
        self.unconstrained_bucket(bucket_name, bucket_account_id)
    }
}


pub trait S3Common: S3 + Clone + Send + Sync + 'static {}

impl<S> S3Common for S
    where S: S3 + Clone + Send + Sync + 'static
{}
