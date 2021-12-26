use rusoto_s3::{
    S3
};
use crate::s3::bucket::{Bucket};

#[derive(Clone)]
pub struct S3Client<S>
    where S: S3Common
{
    #[cfg(not(test))]
    pub(crate) s3_client: S,
    #[cfg(test)]
    pub s3_client: S,
}

impl S3Client<rusoto_s3::S3Client> {
    pub fn new(s3_client: rusoto_s3::S3Client) -> S3Client<rusoto_s3::S3Client> {
        Self { s3_client }
    }
}

impl<S> S3Client<S>
    where S: S3Common
{
    pub fn bucket<
        const READ_CAP: bool,
        const WRITE_CAP: bool,
        const LIST_CAP: bool,
    >(&self, bucket_name: String, bucket_account_id: String) -> Bucket<S, READ_CAP, WRITE_CAP, LIST_CAP> {
        Bucket {
            s3_client: self.clone(),
            bucket_name,
            bucket_account_id,
        }
    }
}

pub trait S3Common: S3 + Clone + Send + Sync + 'static {}

impl<S> S3Common for S
    where S: S3 + Clone + Send + Sync + 'static
{}
