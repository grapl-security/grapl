use async_trait::async_trait;

#[async_trait]
pub trait CompletionHandler {
    type Message;
    type CompletedEvent;

    async fn mark_complete(&self, msg: Self::Message, completed_event: Self::CompletedEvent);
    async fn ack_message(&self, msg: Self::Message);
    async fn ack_all(&self, notify: Option<tokio::sync::oneshot::Sender<()>>);
}
