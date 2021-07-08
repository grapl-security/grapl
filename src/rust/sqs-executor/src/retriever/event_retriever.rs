use async_trait::async_trait;

use crate::errors::CheckedError;
use rust_proto::services::Meta;

#[async_trait]
pub trait PayloadRetriever<T> {
    type Message;
    type Error: CheckedError;
    async fn retrieve_event(&mut self, msg: &Self::Message) -> Result<Option<(Meta, T)>, Self::Error>;
}
