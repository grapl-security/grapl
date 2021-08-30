#![allow(warnings)]

use pin_utils::pin_mut;
use rdkafka::consumer::{CommitMode, Consumer, DefaultConsumerContext};
use rdkafka::util::DefaultRuntime;
use rdkafka::Message;
use tokio_stream::StreamExt;

use grapl_config::env_helpers::FromEnv;

use crate::errors::CheckedError;
use crate::event_consumer::EventConsumer;
use crate::event_decoder::EventDeserializer;
use crate::event_handler::EventHandler;
use crate::event_producer::EventProducer;
use rdkafka::error::KafkaError;
use rdkafka::message::{BorrowedMessage, OwnedMessage};
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::{SemaphorePermit, OwnedSemaphorePermit};
use std::sync::Arc;
use tracing::Instrument;

pub mod errors;
pub mod event_consumer;
pub mod event_decoder;
pub mod event_encoder;
pub mod event_handler;
pub mod event_producer;

#[tracing::instrument(
    skip(
        event_handler,
        consumer,
        producer,
        deserializer,
        serializer,
    ),
    fields(
        src_topic_name = %consumer.topic_name,
        dst_topic_name = %producer.topic_name,
    )
)]
pub async fn service_loop<
    InputEventT:  Send + Sync + 'static,
    OutputEventT:  Send + Sync + 'static,
    EventHandlerErrorT: CheckedError + Send + Sync + 'static,
    EventHandlerT: EventHandler<
            InputEvent = InputEventT,
            OutputEvent = OutputEventT,
            Error = EventHandlerErrorT,
        > + Send + Clone
        + 'static,
    EventDeserializerT: event_decoder::EventDeserializer<InputEvent = InputEventT> + Clone + Send + Sync + 'static,
    EventSerializerT: event_encoder::EventSerializer<OutputEvent = OutputEventT> + Clone + Send + Sync + 'static,
>(
    event_handler: EventHandlerT,
    consumer: &'static EventConsumer<DefaultConsumerContext, DefaultRuntime>,
    producer: EventProducer,
    deserializer: EventDeserializerT,
    serializer: EventSerializerT,
) -> Result<(), Box<dyn std::error::Error>> {
    let sema = Arc::new(tokio::sync::Semaphore::new(std::cmp::max(1, num_cpus::get() - 1)));

    while let Some(message) = consumer.consumer.stream().next().await {
        let message = match message {
            Ok(message) => message,
            Err(e) => {
                tracing::error!(
                    message="Failed to read kafka message",
                    error=%e,
                );
                continue;
            }
        };

        let permit = loop {
            let sema = sema.clone();
            match tokio::time::timeout(
                Duration::from_secs(1), sema.acquire_owned()
            ).await {
                Ok(permit) => break permit.expect("Sempahore can not be closed"),
                Err(e) => {
                    tracing::warn!(message="Failed to acquire semaphore");
                }
            }
        };

        process_message(
            permit,
            message,
            event_handler.clone(),
            &consumer,
            producer.clone(),
            deserializer.clone(),
            serializer.clone(),
        )
        .await;
    }

    Ok(())
}

#[tracing::instrument(
    skip(
        message,
        event_handler,
        consumer,
        producer,
        deserializer,
        serializer,
    ),
    fields(
        offset = message.offset(),
        key = ?message.key()
    )
)]
async fn process_message<
    InputEventT:  Send + Sync + 'static,
    OutputEventT:  Send + Sync + 'static,
    EventHandlerErrorT: CheckedError + Send + Sync + 'static,
    EventHandlerT: EventHandler<
            InputEvent = InputEventT,
            OutputEvent = OutputEventT,
            Error = EventHandlerErrorT,
        >
        + Send + 'static,
    EventDeserializerT: event_decoder::EventDeserializer<InputEvent = InputEventT> + Send + Sync + 'static,
    EventSerializerT: event_encoder::EventSerializer<OutputEvent = OutputEventT> + Send + Sync + 'static,
>(
    permit: OwnedSemaphorePermit,
    message: BorrowedMessage<'static>,
    mut event_handler: EventHandlerT,
    consumer: &'static EventConsumer<DefaultConsumerContext, DefaultRuntime>,
    producer: EventProducer,
    deserializer: EventDeserializerT,
    serializer: EventSerializerT,
) {

    let proc = async move {
        drop(permit);  // forces capture
        let event = match deserializer.decode_event(message.payload().unwrap()) {
            Ok(event) => event,
            Err(e) => {
                tracing::error!(
                message="Failed to decode message payload",
                error=%e,
            );
                return;
            }
        };
        let output = match event_handler.handle_event(event).await {
            Ok(output) => output,
            Err(e) => {
                tracing::error!(
                message="Failed to handle event",
                error=%e,
            );
                return;
            }
        };

        let output = match serializer.encode_event(output) {
            Ok(output) => output,
            Err(e) => {
                tracing::error!(
                message="Failed to encode message payload",
                error=%e,
            );
                return;
            }
        };

        if let Err(e) = producer.emit_event(&output).await {
            tracing::error!(
            message="Failed to emit event",
            error=%e,
        );
            return;
        };

        if let Err(e) = consumer.consumer.commit_message(&message, CommitMode::Sync) {
            tracing::error!(message="Failed to commit message", error=%e);
        };
    };

    tokio::task::spawn(proc.in_current_span());
}

#[cfg(test)]
mod tests {
    use pin_utils::pin_mut;
    use rdkafka::consumer::{CommitMode, Consumer};
    use rdkafka::util::DefaultRuntime;
    use rdkafka::Message;
    use tokio_stream::StreamExt;

    use grapl_config::env_helpers::FromEnv;

    use crate::errors::{CheckedError, Recoverable};
    use crate::event_consumer::EventConsumer;
    use crate::event_decoder::EventDeserializer;
    use crate::event_encoder::EventSerializer;
    use crate::event_handler::EventHandler;
    use crate::event_producer::EventProducer;

    use super::*;

    #[derive(Debug, thiserror::Error)]
    enum ExampleError {
        #[error("JsonError")]
        JsonError(#[from] serde_json::Error),
    }

    impl CheckedError for ExampleError {
        fn error_type(&self) -> Recoverable {
            match self {
                ExampleError::JsonError(_) => Recoverable::Persistent,
            }
        }
    }

    #[derive(Clone)]
    struct JsonDeserializer {}

    impl EventDeserializer for JsonDeserializer {
        type InputEvent = serde_json::Value;
        type Error = ExampleError;

        fn decode_event(&self, event: &[u8]) -> Result<Self::InputEvent, Self::Error> {
            Ok(serde_json::from_slice(&event)?)
        }
    }

    #[derive(Clone)]
    struct JsonSerializer {}

    impl EventSerializer for JsonSerializer {
        type OutputEvent = serde_json::Value;
        type Error = ExampleError;

        fn encode_event(&self, event: Self::OutputEvent) -> Result<Vec<u8>, Self::Error> {
            Ok(serde_json::to_vec(&event)?)
        }
    }

    #[derive(Clone)]
    struct ExampleHandler {}

    #[async_trait::async_trait]
    impl EventHandler for ExampleHandler {
        type InputEvent = serde_json::Value;
        type OutputEvent = serde_json::Value; // This will likely be protos in prod
        type Error = ExampleError;

        async fn handle_event(
            &mut self,
            input: Self::InputEvent,
        ) -> Result<Self::OutputEvent, Self::Error> {
            // just a nop
            Ok(input)
        }
    }

    #[tokio::test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        let event_handler = ExampleHandler {};

        // The consumer must be static since messages reference it.
        lazy_static::lazy_static! {
            static ref consumer: EventConsumer<DefaultConsumerContext, DefaultRuntime>
                = {EventConsumer::from_env()};
        }
        let producer = EventProducer::from_env();

        let deserializer = JsonDeserializer {};
        let serializer = JsonSerializer {};

        service_loop(
            event_handler,
            &consumer,
            producer,
            deserializer,
            serializer,
        ).await;
        // process_message(
        //     &message,
        //     event_handler.as_mut(),
        //     &consumer,
        //     producer.clone(),
        //     deserializer.as_mut(),
        //     serializer.as_mut(),
        // )
        // .await?;
        Ok(())
    }
}
