use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AwsClientError {
    #[error("AwsClientGetError")]
    AwsClientGetError(),
}