mod list;
mod read;
mod write;

use std::collections::HashMap;

pub use list::{
    ListS3,
    *,
};
pub use read::{
    ReadS3,
    *,
};
pub use write::{
    WriteS3,
    *,
};

use crate::s3::client::{
    S3Client,
    S3Common,
};

#[derive(Clone)]
pub struct Bucket<S, const READ_CAP: bool, const WRITE_CAP: bool, const LIST_CAP: bool>
where
    S: S3Common,
{
    #[cfg(test)]
    pub s3_client: S3Client<S>,
    #[cfg(not(test))]
    pub(crate) s3_client: S3Client<S>,
    pub(crate) bucket_name: String,
    pub(crate) bucket_account_id: String,
}

#[derive(Clone)]
pub struct BucketBuilder<S, const READ_CAP: bool, const WRITE_CAP: bool, const LIST_CAP: bool>
    where
        S: S3Common,
{
    #[cfg(test)]
    pub s3_client: S3Client<S>,
    #[cfg(not(test))]
    pub(crate) s3_client: S3Client<S>,
    pub(crate) bucket_name: String,
    pub(crate) bucket_account_id: String,
}


impl<S> BucketBuilder<S, false, false, false>
    where
        S: S3Common,
{
    pub fn new(
        s3_client: S3Client<S>,
        bucket_name: String,
        bucket_account_id: String,
    ) -> Self {
        BucketBuilder {
            s3_client,
            bucket_name,
            bucket_account_id,
        }
    }
}

impl<S, const READ_CAP: bool, const WRITE_CAP: bool, const LIST_CAP: bool> BucketBuilder<S, READ_CAP, WRITE_CAP, LIST_CAP>
    where
        S: S3Common,
{
    pub fn build(self) -> Bucket<S, READ_CAP, WRITE_CAP, LIST_CAP> {
        Bucket {
            s3_client: self.s3_client,
            bucket_name: self.bucket_name,
            bucket_account_id: self.bucket_account_id,
        }
    }
}


impl<S, const WRITE_CAP: bool, const LIST_CAP: bool> BucketBuilder<S, false, WRITE_CAP, LIST_CAP>
    where
        S: S3Common,
{
    pub fn with_read(self) -> BucketBuilder<S, true, WRITE_CAP, LIST_CAP> {
        BucketBuilder {
            s3_client: self.s3_client,
            bucket_name: self.bucket_name,
            bucket_account_id: self.bucket_account_id,
        }
    }
}


impl<S, const READ_CAP: bool, const LIST_CAP: bool> BucketBuilder<S, READ_CAP, false, LIST_CAP>
    where
        S: S3Common,
{
    pub fn with_write(self) -> BucketBuilder<S, READ_CAP, true, LIST_CAP> {
        BucketBuilder {
            s3_client: self.s3_client,
            bucket_name: self.bucket_name,
            bucket_account_id: self.bucket_account_id,
        }
    }
}


impl<S, const READ_CAP: bool, const WRITE_CAP: bool> BucketBuilder<S, READ_CAP, WRITE_CAP, false>
    where
        S: S3Common,
{
    pub fn with_list(self) -> BucketBuilder<S, READ_CAP, WRITE_CAP, true> {
        BucketBuilder {
            s3_client: self.s3_client,
            bucket_name: self.bucket_name,
            bucket_account_id: self.bucket_account_id,
        }
    }
}



pub trait Metadata {
    fn merge_into(self, metadata: &mut HashMap<String, String>);
}

impl Metadata for (String, String) {
    fn merge_into(self, metadata: &mut HashMap<String, String>) {
        metadata.insert(self.0, self.1);
    }
}

impl Metadata for HashMap<String, String> {
    fn merge_into(self, metadata: &mut HashMap<String, String>) {
        metadata.extend(self.into_iter());
    }
}
