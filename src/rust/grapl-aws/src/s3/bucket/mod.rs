mod read;
mod write;
mod list;

pub use read::*;
pub use write::*;
pub use list::*;

use std::collections::HashMap;
use crate::s3::client::{S3Client, S3Common};


#[derive(Clone)]
pub struct Bucket<S>
    where S: S3Common
{
    #[cfg(test)]
    pub s3_client: S3Client<S>,
    #[cfg(not(test))]
    pub(crate) s3_client: S3Client<S>,
    pub(crate) bucket_name: String,
    pub(crate) bucket_account_id: String,
}


pub trait MetaData {
    fn merge_into(self, metadata: &mut HashMap<String, String>);
}

impl MetaData for (String, String) {
    fn merge_into(self, metadata: &mut HashMap<String, String>) {
        metadata.insert(self.0, self.1);
    }
}

impl MetaData for HashMap<String, String> {
    fn merge_into(self, metadata: &mut HashMap<String, String>) {
        metadata.extend(self.into_iter());
    }
}
