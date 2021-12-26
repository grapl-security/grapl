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
