use async_trait::async_trait;

use crate::event_processor::EventProcessorActor;

#[async_trait]
pub trait Consumer<M>
where
    M: Send + Clone + Sync + 'static,
{
    async fn get_next_event(&self, event_processor: EventProcessorActor<M>);
}
