use rusoto_s3::S3;

use crate::s3::bucket::Bucket;

#[derive(Clone)]
pub struct S3Client<S>
where
    S: S3Common,
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
where
    S: S3Common,
{
    /// `bucket` returns a reference to an S3 Bucket with the type expressing your
    /// capabilities for operating on that bucket.
    ///
    /// For example, if you're working with a bucket that you can only read from, you should
    /// construct a bucket with read_only capabilities.
    ///
    /// The three capabilities we express currently are Read, Write, List
    ///
    /// Capabilities in the type system do not necessarily correspond to capabilities in IAM. It
    /// is up to the programmer to ensure that they have the proper IAM permissions.
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///   let s3_client = S3Client::new(rusoto_s3::S3Client::from_env());
    ///
    ///
    ///   let bucket: Bucket<_, true, false, false> = s3_client.bucket(
    ///     "local-test-read-objects-bucket".to_owned(),
    ///     "000000000000".to_owned(),
    ///   );
    ///
    ///   let object = bucket.get_object("path").await?;
    ///
    ///   // The code below won't compile because our bucket does not have 'write' capabilities
    ///   // bucket.put_object("path", "value").await?;
    /// }
    /// ```
    pub fn bucket<const READ_CAP: bool, const WRITE_CAP: bool, const LIST_CAP: bool>(
        &self,
        bucket_name: String,
        bucket_account_id: String,
    ) -> Bucket<S, READ_CAP, WRITE_CAP, LIST_CAP> {
        Bucket {
            s3_client: self.clone(),
            bucket_name,
            bucket_account_id,
        }
    }
}

pub trait S3Common: S3 + Clone + Send + Sync + 'static {}

impl<S> S3Common for S where S: S3 + Clone + Send + Sync + 'static {}
