#[async_trait::async_trait]
pub trait StreamProcessor {
    type InputEvent;
    type OutputEvent;
    type Error: std::error::Error;

    async fn handle_event(
        &mut self,
        input: Self::InputEvent,
    ) -> Result<Self::OutputEvent, Self::Error>;
}
