use std::fmt::Debug;
use async_trait::async_trait;
use crate::errors::CheckedError;
#[async_trait]
pub trait EventHandler {
    type InputEvent;
    type OutputEvent: Clone + Send + Sync + 'static;
    type Error: Debug + CheckedError + Send + Sync + 'static;

    async fn handle_event(
        &mut self,
        input: Self::InputEvent,
    ) -> Result<Self::OutputEvent, Result<Self::OutputEvent, Self::Error>>;
}
