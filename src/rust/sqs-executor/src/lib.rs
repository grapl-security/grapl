use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::FutureExt;
use rusoto_core::RusotoError;
use rusoto_s3::{S3, S3Client};
use rusoto_sqs::{DeleteMessageError, DeleteMessageRequest, Message as SqsMessage, ReceiveMessageError, ReceiveMessageRequest};
use rusoto_sqs::{Sqs, SqsClient};
use thiserror::Error;
use tracing::{info, error, warn};

use event_emitter::EventEmitter;
use event_handler::EventHandler;
use event_retriever::S3PayloadRetriever;
use s3_event_emitter::S3EventEmitter;

use crate::completion_event_serializer::CompletionEventSerializer;
use crate::errors::{CheckedError, ExecutorError, Recoverable};
use crate::event_decoder::PayloadDecoder;
use crate::event_retriever::PayloadRetriever;
use crate::event_handler::CompletedEvents;
use tracing::debug;

pub mod cache;
pub mod redis_cache;
pub mod event_retriever;
pub mod event_decoder;
pub mod event_handler;
pub mod event_emitter;
pub mod s3_event_emitter;
pub mod completion_event_serializer;
pub mod key_creator;
pub mod errors;

// Message - Contains information necessary to retrieve payload
// Payload - Encoded set of Events
// Event - Individual unit to process, each Event must have an identity

// Sqs Message -> S3 Payload -> Sysmon Event
//

pub async fn get_message<SqsT>(queue_url: String, sqs_client: SqsT) -> Result<Vec<SqsMessage>, RusotoError<ReceiveMessageError>>
    where SqsT: Sqs + Clone + Send + Sync + 'static,
{
    let messages = sqs_client.receive_message(ReceiveMessageRequest {
        max_number_of_messages: Some(10),
        queue_url,
        visibility_timeout: Some(30),
        wait_time_seconds: Some(20),
        ..Default::default()
    });

    let messages = tokio::time::timeout(
        std::time::Duration::from_secs(21),
        messages,
    );
    let messages = messages.await.expect("timeout")?
    .messages.unwrap_or_else(|| vec![]);

    Ok(messages)
}

impl CheckedError for RusotoError<DeleteMessageError> {
    fn error_type(&self) -> Recoverable {
        match self {
            RusotoError::Service(DeleteMessageError::InvalidIdFormat(_)) => {
                Recoverable::Persistent
            }
            RusotoError::Service(DeleteMessageError::ReceiptHandleIsInvalid(_)) => {
                Recoverable::Persistent
            }
            RusotoError::HttpDispatch(e) => {
                Recoverable::Transient
            }
            RusotoError::Credentials(_) => {
                // todo: Reasonable
                Recoverable::Transient
            }
            RusotoError::Validation(_) => {
                // todo: Reasonable?
                Recoverable::Transient
            }
            RusotoError::ParseError(_) => {
                // todo: Is this reasonable?
                Recoverable::Transient
            }
            RusotoError::Unknown(_) => {
                Recoverable::Transient
            }
            RusotoError::Blocking => {
                Recoverable::Transient
            }
        }
    }
}

fn delete_message<SqsT>(
    sqs_client: SqsT,
    queue_url: String,
    receipt_handle: String,
) -> tokio::task::JoinHandle<()>
    where SqsT: Sqs + Clone + Send + Sync + 'static,
{
    tokio::task::spawn(async move {
        for _ in 0..5u8 {
            match sqs_client.clone().delete_message(
                DeleteMessageRequest {
                    queue_url: queue_url.clone(),
                    receipt_handle: receipt_handle.clone(),
                }
            )
                .await {
                Ok(_) => {
                    debug!("Deleted message: {}", receipt_handle.clone());
                    return
                },
                Err(e) => {
                    error!("Failed to delete_message with: {:?} {:?}", e, e.error_type());
                    if let Recoverable::Persistent = e.error_type() {
                        return;
                    }
                }
            }
        }
    })
}

async fn process_message<
    CacheT,
    SInit,
    SqsT,
    DecoderT,
    EventHandlerT,
    InputEventT,
    OutputEventT,
    HandlerErrorT,
    SerializerErrorT,
    S3ClientT,
    F,
    OnEmission,
    EmissionResult,
    CompletionEventSerializerT,
>(
    next_message: SqsMessage,
    queue_url: String,
    cache: &mut CacheT,
    sqs_client: SqsT,
    event_handler: &mut EventHandlerT,
    s3_payload_retriever: &mut S3PayloadRetriever<S3ClientT, SInit, DecoderT, InputEventT>,
    s3_emitter: &mut S3EventEmitter<S3ClientT, F, OnEmission, EmissionResult>,
    serializer: &mut CompletionEventSerializerT,
)
    where
        CacheT: crate::cache::Cache + Clone + Send + Sync + 'static,
        SInit: (Fn(String) -> S3ClientT) + Clone + Send + Sync + 'static,
        SqsT: Sqs + Clone + Send + Sync + 'static,
        DecoderT: PayloadDecoder<InputEventT> + Clone + Send + 'static,
        InputEventT: Send,
        EventHandlerT: EventHandler<
            InputEvent=InputEventT,
            OutputEvent=OutputEventT,
            Error=HandlerErrorT,
        >,
        OutputEventT: Clone + Send + Sync + 'static,
        HandlerErrorT: CheckedError + Debug + Send + Sync + 'static,
        SerializerErrorT: Error + Debug + Send + Sync + 'static,
        S3ClientT: S3 + Clone + Send + Sync + 'static,
        F: Fn(&[u8]) -> String + Send + Sync,
        EmissionResult: Future<Output=Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
        OnEmission: Fn(String, String) -> EmissionResult + Send + Sync + 'static,
        CompletionEventSerializerT: CompletionEventSerializer<
            CompletedEvent=OutputEventT,
            Output=Vec<u8>,
            Error=SerializerErrorT,
        >
{
    info!("Retrieving payload");
    let payload = s3_payload_retriever.retrieve_event(&next_message).await;

    let events = match payload {
        Ok(Some(events)) => events,
        Ok(None) => {
            delete_message(
                sqs_client.clone(),
                queue_url.to_owned(),
                next_message.receipt_handle.expect("missing receipt_handle"),
            );

            return;
        }
        Err(e) => {
            // If this thing's persistent, let's just delete the message
            // todo: It should actually be moved to the dead-letter, not deleted
            if let Recoverable::Persistent = e.error_type() {
                delete_message(
                    sqs_client.clone(),
                    queue_url.to_owned(),
                    next_message.receipt_handle.expect("missing receipt_handle"),
                );
            }
            return;
        }
    };

    // todo: We can lift this
    let mut completed = CompletedEvents::default();

    // completed.clear();
    let processing_result = event_handler.handle_event(
        events, &mut completed,
    ).await;


    match processing_result {
        Ok(total) => {
            // encode event
            let event = serializer
                .serialize_completed_events(&[total])
                .expect("Serializing failed");
            // emit event
            // todo: we should retry event emission
            s3_emitter.emit_event(event).await
                .expect("Failed to emit event");

            for identity in completed.identities.drain(..) {
                if let Err(e) = cache.store(identity).await {
                    warn!("Failed to store identity in cache: {:?}", e);
                }
            }
            // ack the message
            delete_message(
                sqs_client.clone(),
                queue_url.to_owned(),
                next_message.receipt_handle.expect("missing receipt_handle"),
            ).await;
        }
        Err(Ok((partial, e))) => {
            error!("Processing failed with: {:?}", e);
            let event = serializer
                .serialize_completed_events(&[partial])
                .expect("Serializing failed");
            // emit event
            // todo: we should retry event emission
            s3_emitter.emit_event(event).await
                .expect("Failed to emit event");

            for identity in completed.identities.drain(..) {
                if let Err(e) = cache.store(identity).await {
                    warn!("Failed to store identity in cache: {:?}", e);
                }
            }

            if let Recoverable::Persistent = e.error_type() {
                // todo: We should move this to the deadletter directly
                delete_message(
                    sqs_client.clone(),
                    queue_url.to_owned(),
                    next_message.receipt_handle.expect("missing receipt_handle"),
                );
            }
        }
        Err(Err(e)) => {
            error!("Handler failed with: {:?} Recoverable: {:?}", e, e.error_type());
            if let Recoverable::Persistent = e.error_type() {
                // todo: We should move this to the deadletter directly
                delete_message(
                    sqs_client.clone(),
                    queue_url.to_owned(),
                    next_message.receipt_handle.expect("missing receipt_handle"),
                );
            }
            // should we retry? idk
            // otherwise we can just do nothing
        }
    }
}


async fn _process_loop<
    CacheT,
    SInit,
    SqsT,
    DecoderT,
    EventHandlerT,
    InputEventT,
    OutputEventT,
    HandlerErrorT,
    SerializerErrorT,
    S3ClientT,
    F,
    OnEmission,
    EmissionResult,
    CompletionEventSerializerT,
>(
    queue_url: String,
    cache: &mut [CacheT; 10],
    sqs_client: SqsT,
    event_handler: &mut [EventHandlerT; 10],
    s3_payload_retriever: &mut [S3PayloadRetriever<S3ClientT, SInit, DecoderT, InputEventT>; 10],
    s3_emitter: &mut [S3EventEmitter<S3ClientT, F, OnEmission, EmissionResult>; 10],
    serializer: &mut [CompletionEventSerializerT; 10],
)
    where
        CacheT: crate::cache::Cache + Clone + Send + Sync + 'static,
        SInit: (Fn(String) -> S3ClientT) + Clone + Send + Sync + 'static,
        SqsT: Sqs + Clone + Send + Sync + 'static,
        DecoderT: PayloadDecoder<InputEventT> + Clone + Send + 'static,
        InputEventT: Send,
        EventHandlerT: EventHandler<
            InputEvent=InputEventT,
            OutputEvent=OutputEventT,
            Error=HandlerErrorT,
        >,
        OutputEventT: Clone + Send + Sync + 'static,
        HandlerErrorT: CheckedError + Debug + Send + Sync + 'static,
        SerializerErrorT: Error + Debug + Send + Sync + 'static,
        S3ClientT: S3 + Clone + Send + Sync + 'static,
        F: Fn(&[u8]) -> String + Send + Sync,
        EmissionResult: Future<Output=Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
        OnEmission: Fn(String, String) -> EmissionResult + Send + Sync + 'static,
        CompletionEventSerializerT: CompletionEventSerializer<
            CompletedEvent=OutputEventT,
            Output=Vec<u8>,
            Error=SerializerErrorT,
        >
{
    loop {
        info!("Retrieving messages");
        println!("Retrieving messages");
        let message_batch = get_message(
            queue_url.to_string(),
            sqs_client.clone(),
        ).await.expect("failed to get messages");
        info!("Received {} messages", message_batch.len());
        println!("Received {} messages", message_batch.len());
        if message_batch.is_empty() {
            continue
        }
        // We can't parallelize this because of the shared mutable state
        // of the retriever, emitter, and serializer

        // We could just preallocate multiple of each - like 10 of each,
        // and then just pick one of them at a time.

        // let mut tasks = Vec::with_capacity(message_batch.len());
        let combos = message_batch.into_iter()
            .zip(&mut *event_handler)
            .zip(&mut *s3_payload_retriever)
            .zip(&mut *s3_emitter)
            .zip(&mut *serializer)
            .zip(&mut *cache)
            ;
        for combo in combos {
            let ((((((next_message, event_handler), s3_payload_retriever)), s3_emitter), serializer), cache) = combo;
            process_message(
                next_message,
                queue_url.clone(),
                cache,
                sqs_client.clone(),
                event_handler,
                s3_payload_retriever,
                s3_emitter,
                serializer,
            ).await;
        }
        // let results = futures::future::join_all(tasks).await;
    }
}

pub async fn process_loop<
    CacheT,
    SInit,
    SqsT,
    DecoderT,
    EventHandlerT,
    InputEventT,
    OutputEventT,
    HandlerErrorT,
    SerializerErrorT,
    S3ClientT,
    F,
    OnEmission,
    EmissionResult,
    CompletionEventSerializerT,
>(
    queue_url: String,
    cache: &mut [CacheT; 10],
    sqs_client: SqsT,
    event_handler: &mut [EventHandlerT; 10],
    s3_payload_retriever: &mut [S3PayloadRetriever<S3ClientT, SInit, DecoderT, InputEventT>; 10],
    s3_emitter: &mut [S3EventEmitter<S3ClientT, F, OnEmission, EmissionResult>; 10],
    serializer: &mut [CompletionEventSerializerT; 10],
)
    where
        CacheT: crate::cache::Cache + Clone + Send + Sync + 'static,
        SInit: (Fn(String) -> S3ClientT) + Clone + Send + Sync + 'static,
        SqsT: Sqs + Clone + Send + Sync + 'static,
        DecoderT: PayloadDecoder<InputEventT> + Clone + Send + 'static,
        InputEventT: Send,
        EventHandlerT: EventHandler<
            InputEvent=InputEventT,
            OutputEvent=OutputEventT,
            Error=HandlerErrorT,
        >,
        OutputEventT: Clone + Send + Sync + 'static,
        HandlerErrorT: CheckedError + Debug + Send + Sync + 'static,
        SerializerErrorT: Error + Debug + Send + Sync + 'static,
        S3ClientT: S3 + Clone + Send + Sync + 'static,
        F: Fn(&[u8]) -> String + Send + Sync,
        EmissionResult: Future<Output=Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
        OnEmission: Fn(String, String) -> EmissionResult + Send + Sync + 'static,
        CompletionEventSerializerT: CompletionEventSerializer<
            CompletedEvent=OutputEventT,
            Output=Vec<u8>,
            Error=SerializerErrorT,
        >
{
    loop {
        debug!("Outer process loop");
        let f = _process_loop(
            queue_url.clone(),
            cache,
            sqs_client.clone(),
            event_handler,
            s3_payload_retriever,
            s3_emitter,
            serializer,
        );
        let f = AssertUnwindSafe(f);
        if let Err(e) = f.catch_unwind().await {
            if let Some(e) = e.downcast_ref::<Box<dyn std::fmt::Debug>>() {
                error!("Processing loop failed {:?}", e);
            } else {
                error!("Unexpected error");
            }
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            // todo: maybe a sleep/ backoff with random jitter?
        }
    }
}

pub fn time_based_key_fn(_event: &[u8]) -> String {
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!("{}/{}-{}", cur_day, cur_ms, uuid::Uuid::new_v4())
}
