use async_trait::async_trait;

use crate::errors::CheckedError;

#[async_trait]
pub trait EventEmitter {
    type Event;
    type Error: CheckedError + std::fmt::Debug + Send;
    async fn emit_event(&mut self, completed_events: Vec<Self::Event>) -> Result<(), Self::Error>;
}
