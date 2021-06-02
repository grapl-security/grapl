use async_trait::async_trait;

#[async_trait]
pub trait EventEmitter {
    type Event;
    type Error: std::fmt::Debug + Send;
    async fn emit_event(&mut self, completed_events: Vec<Self::Event>) -> Result<(), Self::Error>;
}
