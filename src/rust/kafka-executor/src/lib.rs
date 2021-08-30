use std::pin::Pin;

use pin_utils::pin_mut;
use rdkafka::{
    consumer::{
        CommitMode,
        Consumer,
        DefaultConsumerContext,
    },
    message::BorrowedMessage,
    util::DefaultRuntime,
    Message,
};
use tokio_stream::StreamExt;

use crate::{
    event_consumer::KafkaConsumer,
    event_handler::StreamProcessor,
    event_producer::KafkaProducer,
};

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
    InputEventT: Send + Sync + 'static,
    OutputEventT: Send + Sync + 'static,
    StreamProcessorErrorT: std::error::Error + Send + Unpin + 'static,
    StreamProcessorT: StreamProcessor<
            InputEvent = InputEventT,
            OutputEvent = OutputEventT,
            Error = StreamProcessorErrorT,
        > + Send
        + Unpin
        + 'static,
    DeserializerT: event_decoder::Deserializer<InputEvent = InputEventT> + Send + Unpin + 'static,
    SerializerT: event_encoder::Serializer<OutputEvent = OutputEventT> + Send + Unpin + 'static,
>(
    event_handler: StreamProcessorT,
    consumer: KafkaConsumer<DefaultConsumerContext, DefaultRuntime>,
    producer: KafkaProducer,
    deserializer: DeserializerT,
    serializer: SerializerT,
) {
    pin_mut!(event_handler);
    pin_mut!(deserializer);
    pin_mut!(serializer);
    let producer = Pin::new(&producer);

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
        process_message(
            message,
            event_handler.as_mut(),
            &consumer,
            producer.as_ref(),
            deserializer.as_mut(),
            serializer.as_mut(),
        )
        .await;
    }
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
    InputEventT: Send + 'static,
    OutputEventT: Send + 'static,
    StreamProcessorErrorT: std::error::Error + Send + 'static,
    StreamProcessorT: StreamProcessor<
            InputEvent = InputEventT,
            OutputEvent = OutputEventT,
            Error = StreamProcessorErrorT,
        > + Send
        + Unpin
        + 'static,
    DeserializerT: event_decoder::Deserializer<InputEvent = InputEventT> + Unpin + Send + 'static,
    SerializerT: event_encoder::Serializer<OutputEvent = OutputEventT> + Unpin + Send + 'static,
>(
    message: BorrowedMessage<'_>,
    mut event_handler: Pin<&mut StreamProcessorT>,
    consumer: &KafkaConsumer<DefaultConsumerContext, DefaultRuntime>,
    producer: Pin<&KafkaProducer>,
    mut deserializer: Pin<&mut DeserializerT>,
    mut serializer: Pin<&mut SerializerT>,
) {
    // let envelope = message.payload().unwrap();
    // let envelope = Envelope::from

    let event = match deserializer.deserialize(message.payload().unwrap()) {
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

    let output = match serializer.serialize(output) {
        Ok(output) => output,
        Err(e) => {
            tracing::error!(
                message="Failed to encode message payload",
                error=%e,
            );
            return;
        }
    };

    if let Err(e) = producer.produce(&output).await {
        tracing::error!(
            message="Failed to emit event",
            error=%e,
        );
        return;
    };

    if let Err(e) = consumer.consumer.commit_message(&message, CommitMode::Sync) {
        tracing::error!(message="Failed to commit message", error=%e);
    };
}

#[cfg(test)]
mod tests {
    use futures::{
        stream::FuturesUnordered,
        StreamExt,
    };
    use grapl_config::env_helpers::FromEnv;

    use super::*;
    use crate::{
        event_consumer::KafkaConsumer,
        event_decoder::Deserializer,
        event_encoder::Serializer,
        event_handler::StreamProcessor,
        event_producer::KafkaProducer,
    };

    #[derive(Debug, thiserror::Error)]
    enum ExampleError {
        #[error("JsonError")]
        JsonError(#[from] serde_json::Error),
    }

    #[derive(Clone)]
    struct JsonDeserializer {}

    impl Deserializer for JsonDeserializer {
        type InputEvent = serde_json::Value;
        type Error = ExampleError;

        fn deserialize(&mut self, event: &[u8]) -> Result<Self::InputEvent, Self::Error> {
            Ok(serde_json::from_slice(event)?)
        }
    }

    #[derive(Clone)]
    struct JsonSerializer {}

    impl Serializer for JsonSerializer {
        type OutputEvent = serde_json::Value;
        type Error = ExampleError;

        fn serialize(&mut self, event: Self::OutputEvent) -> Result<Vec<u8>, Self::Error> {
            Ok(serde_json::to_vec(&event)?)
        }
    }

    #[derive(Clone)]
    struct ExampleHandler {}

    #[async_trait::async_trait]
    impl StreamProcessor for ExampleHandler {
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

        (0..num_cpus::get())
            .map(|_| {
                tokio::spawn(service_loop(
                    event_handler.clone(),
                    KafkaConsumer::from_env(),
                    KafkaProducer::from_env(),
                    JsonDeserializer {},
                    JsonSerializer {},
                ))
            })
            .collect::<FuturesUnordered<_>>()
            .for_each(|_| async { () })
            .await;
        Ok(())
    }
}
