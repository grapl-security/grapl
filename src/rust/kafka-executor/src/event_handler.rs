use crate::errors::CheckedError;

#[async_trait::async_trait]
pub trait EventHandler {
    type InputEvent;
    type OutputEvent;
    type Error: CheckedError;

    async fn handle_event(
        &mut self,
        input: Self::InputEvent,
    ) -> Result<Self::OutputEvent, Self::Error>;
}
