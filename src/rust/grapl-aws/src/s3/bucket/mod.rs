mod read;
mod write;
mod list;

pub use read::*;
pub use read::ReadS3;
pub use write::*;
pub use write::WriteS3;
pub use list::*;
pub use list::ListS3;

use std::collections::HashMap;
use crate::s3::client::{S3Client, S3Common};


#[derive(Clone)]
pub struct Bucket<
    S,
    const READ_CAP: bool,
    const WRITE_CAP: bool,
    const LIST_CAP: bool,
>
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
