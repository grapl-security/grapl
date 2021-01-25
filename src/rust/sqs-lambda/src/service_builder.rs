use crate::completion_handler::CompletionHandler;
use crate::consumer::Consumer;
use crate::event_processor::EventProcessorActor;

pub struct ServiceBuilder<
    ConsumerT,
    TriggerT, // SqsMessage
    CompletedEventT,
    CompletionHandlerT,
> where
    ConsumerT: Consumer<TriggerT>,
    TriggerT: Send + Clone + Sync + 'static,
    CompletedEventT: Send + Clone + Sync + 'static,
    CompletionHandlerT: CompletionHandler<Message = TriggerT, CompletedEvent = CompletedEventT>,
{
    _trigger_consumer: ConsumerT,
    _event_processor: EventProcessorActor<TriggerT>,
    _completion_handler: CompletionHandlerT,
}
