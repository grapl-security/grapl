#![allow(
    dead_code,
    non_camel_case_types,
    non_upper_case_globals,
    unreachable_code,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_variables,
)]
pub mod retriever;








use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::io::Stdout;
use std::panic::AssertUnwindSafe;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::FutureExt;
use rusoto_core::RusotoError;
use rusoto_s3::{S3Client, S3};
use rusoto_sqs::{
    DeleteMessageError, DeleteMessageRequest, Message as SqsMessage, ReceiveMessageError,
    ReceiveMessageRequest, SendMessageError as InnerSendMessageError, SendMessageRequest,
};
use rusoto_sqs::{Sqs, SqsClient};
use thiserror::Error;
use tokio::task::{JoinError, JoinHandle};
use tokio::time::Elapsed;
use tracing::debug;
use tracing::{error, info, warn};

use event_emitter::EventEmitter;
use event_handler::EventHandler;
use event_retriever::S3PayloadRetriever;
use grapl_observe::metric_reporter::{MetricReporter, tag};
use grapl_observe::timers::{time_fut_ms, TimedFutureExt};
use s3_event_emitter::S3EventEmitter;

use crate::cache::{Cache, CacheResponse};
use crate::completion_event_serializer::CompletionEventSerializer;
use crate::errors::{CheckedError, ExecutorError, Recoverable};
use crate::event_decoder::PayloadDecoder;
use crate::event_handler::CompletedEvents;
use crate::event_retriever::PayloadRetriever;
use crate::event_status::EventStatus;
use crate::s3_event_emitter::OnEventEmit;

pub mod cache;
pub mod completion_event_serializer;
pub mod errors;
pub mod event_decoder;
pub mod event_emitter;
pub mod event_handler;
pub use retriever::event_retriever;
pub use retriever::s3_event_retriever;
use crate::sqs_timeout_manager::cleanup_message;

pub mod event_status;
pub mod key_creator;
pub mod redis_cache;
pub mod rusoto_helpers;
pub mod s3_event_emitter;
pub mod sqs_timeout_manager;

pub async fn make_ten<F, T>(f: F) -> [T; 10]
where
    F: Future<Output = T>,
    T: Clone,
{
    let t = f.await;

    [
        t.clone(),
        t.clone(),
        t.clone(),
        t.clone(),
        t.clone(),
        t.clone(),
        t.clone(),
        t.clone(),
        t.clone(),
        t,
    ]
}

async fn cache_completed<CacheT>(cache: &mut CacheT, completed: &mut CompletedEvents)
where
    CacheT: Cache,
{
    let mut cache_stores = Vec::with_capacity(completed.identities.len());
    for (identity, status) in completed.identities.drain(..) {
        let mut cache = cache.clone();
        let cache_store = async move {
            match status {
                EventStatus::Success | EventStatus::Failure => {
                    if let Err(e) = cache.store(identity).await {
                        warn!("Failed to store identity in cache: {:?}", e);
                    }
                }
                _ => (),
            }
        };
        cache_stores.push(cache_store);
    }
    futures::future::join_all(cache_stores).await;
}

async fn process_message<
    CacheT,
    SInit,
    SqsT,
    DecoderT,
    DecoderErrorT,
    EventHandlerT,
    InputEventT,
    OutputEventT,
    HandlerErrorT,
    SerializerErrorT,
    S3ClientT,
    OnEmit,
    OnEmitError,
    F,
    CompletionEventSerializerT,
>(
    next_message: SqsMessage,
    queue_url: String,
    dead_letter_queue_url: String,
    cache: &mut CacheT,
    sqs_client: SqsT,
    event_handler: &mut EventHandlerT,
    s3_payload_retriever: &mut S3PayloadRetriever<
        S3ClientT,
        SInit,
        DecoderT,
        InputEventT,
        DecoderErrorT,
    >,
    s3_emitter: &mut S3EventEmitter<S3ClientT, F, OnEmit, OnEmitError>,
    serializer: &mut CompletionEventSerializerT,
    mut metric_reporter: MetricReporter<Stdout>,
) where
    CacheT: crate::cache::Cache + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S3ClientT) + Clone + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
    DecoderT: PayloadDecoder<InputEventT, DecoderError = DecoderErrorT> + Clone + Send + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
    InputEventT: Send,
    EventHandlerT:
        EventHandler<InputEvent = InputEventT, OutputEvent = OutputEventT, Error = HandlerErrorT>,
    OutputEventT: Clone + Send + Sync + 'static,
    HandlerErrorT: CheckedError + Debug + Send + Sync + 'static,
    SerializerErrorT: Error + Debug + Send + Sync + 'static,
    S3ClientT: S3 + Clone + Send + Sync + 'static,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
    OnEmit: Clone + OnEventEmit<Error = OnEmitError> + Send + Sync + 'static,
    OnEmitError: CheckedError + Send,
    CompletionEventSerializerT: CompletionEventSerializer<
        CompletedEvent = OutputEventT,
        Output = Vec<u8>,
        Error = SerializerErrorT,
    >,
{
    if let Ok(CacheResponse::Hit) = cache.get(next_message.message_id.clone().unwrap().into_bytes()).await {
        info!("Message has already been processed: {:?}", next_message.message_id);
        rusoto_helpers::delete_message(
            sqs_client.clone(),
            queue_url.to_owned(),
            next_message.receipt_handle.expect("missing receipt_handle"),
            metric_reporter,
        );
        return
    }
    info!("Retrieving payload from: {:?}", next_message.message_id);
    let receipt_handle = next_message.receipt_handle.as_ref().expect("missing receipt_handle").to_owned();
    let msg_handle = cleanup_message(sqs_client.clone(), receipt_handle.clone(), queue_url.clone(), 30,);
    let payload = s3_payload_retriever.retrieve_event(&next_message).await;

    let events = match payload {
        Ok(Some(events)) => events,
        Ok(None) => {
            rusoto_helpers::delete_message(
                sqs_client.clone(),
                queue_url.to_owned(),
                receipt_handle,
                metric_reporter.clone(),
            );
            return;
        }
        Err(e) => {
            error!("Failed to retrieve payload with: {:?}", e);
            if let Recoverable::Persistent = e.error_type() {
                rusoto_helpers::move_to_dead_letter(
                    sqs_client.clone(),
                    next_message.body.as_ref().unwrap(),
                    dead_letter_queue_url,
                    queue_url.to_owned(),
                    receipt_handle,
                    metric_reporter.clone(),
                )
                .await;
            }
            return;
        }
    };

    // todo: We can lift this
    let mut completed = CompletedEvents::default();

    // completed.clear();

    let processing_result = async {
        let (processing_result, ms) = event_handler.handle_event(events, &mut completed)
            .timed().await;
        metric_reporter.histogram(
            "event_handler.handle_event",
            ms as f64,
            &[tag("success", processing_result.is_ok())]
        );
        processing_result
    }.await;

    match processing_result {
        Ok(total) => {
            // encode event
            let event = serializer
                .serialize_completed_events(&[total])
                .expect("Serializing failed");
            // emit event
            // todo: we should retry event emission
            s3_emitter
                .emit_event(event)
                .await
                .expect("Failed to emit event");

            cache.store(next_message.message_id.clone().unwrap().into_bytes()).await;
            cache_completed(cache, &mut completed).await;
            // ack the message - we could probably not block on this

            msg_handle.stop();
            rusoto_helpers::delete_message(
                sqs_client.clone(),
                queue_url.to_owned(),
                receipt_handle,
                metric_reporter.clone(),
            )
            .await;
        }
        Err(Ok((partial, e))) => {
            error!(
                "Handler failed with: {:?} Recoverable: {:?}",
                e,
                e.error_type()
            );
            let event = serializer
                .serialize_completed_events(&[partial])
                .expect("Serializing failed");
            // emit event
            // todo: we should retry event emission
            s3_emitter
                .emit_event(event)
                .await
                .expect("Failed to emit event");

            cache_completed(cache, &mut completed).await;

            if let Recoverable::Persistent = e.error_type() {

                msg_handle.stop();
                rusoto_helpers::move_to_dead_letter(
                    sqs_client.clone(),
                    next_message.body.as_ref().unwrap(),
                    dead_letter_queue_url,
                    queue_url.to_owned(),
                    receipt_handle,
                    metric_reporter.clone(),
                )
                .await;
            }
        }
        Err(Err(e)) => {
            error!(
                "Handler failed with: {:?} Recoverable: {:?}",
                e,
                e.error_type()
            );
            if let Recoverable::Persistent = e.error_type() {
                msg_handle.stop();
                rusoto_helpers::move_to_dead_letter(
                    sqs_client.clone(),
                    next_message.body.as_ref().unwrap(),
                    dead_letter_queue_url,
                    queue_url.to_owned(),
                    receipt_handle,
                    metric_reporter.clone(),
                )
                .await;
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
    DecoderErrorT,
    EventHandlerT,
    InputEventT,
    OutputEventT,
    HandlerErrorT,
    SerializerErrorT,
    S3ClientT,
    F,
    OnEmit,
    OnEmitError,
    CompletionEventSerializerT,
>(
    queue_url: String,
    dead_letter_queue_url: String,
    cache: &mut [CacheT; 10],
    sqs_client: SqsT,
    event_handler: &mut [EventHandlerT; 10],
    s3_payload_retriever: &mut [S3PayloadRetriever<S3ClientT, SInit, DecoderT, InputEventT, DecoderErrorT>;
             10],
    s3_emitter: &mut [S3EventEmitter<S3ClientT, F, OnEmit, OnEmitError>; 10],
    serializer: &mut [CompletionEventSerializerT; 10],
    mut metric_reporter: MetricReporter<Stdout>,
) where
    CacheT: crate::cache::Cache + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S3ClientT) + Clone + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
    DecoderT: PayloadDecoder<InputEventT, DecoderError = DecoderErrorT> + Clone + Send + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
    InputEventT: Send,
    EventHandlerT:
        EventHandler<InputEvent = InputEventT, OutputEvent = OutputEventT, Error = HandlerErrorT>,
    OutputEventT: Clone + Send + Sync + 'static,
    HandlerErrorT: CheckedError + Debug + Send + Sync + 'static,
    SerializerErrorT: Error + Debug + Send + Sync + 'static,
    S3ClientT: S3 + Clone + Send + Sync + 'static,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
    OnEmit: Clone + OnEventEmit<Error = OnEmitError> + Send + Sync + 'static,
    OnEmitError: CheckedError + Send,
    CompletionEventSerializerT: CompletionEventSerializer<
        CompletedEvent = OutputEventT,
        Output = Vec<u8>,
        Error = SerializerErrorT,
    >,
{
    let mut i = 1;
    loop {
        if i >= 15 {
            i = 2;
        }

        info!("Retrieving messages from {}", queue_url);
        println!("Retrieving messages from {}", queue_url);
        let message_batch = rusoto_helpers::get_message(
            queue_url.to_string(),
            sqs_client.clone(),
            &mut metric_reporter,
        )
        .await;

        let message_batch = match message_batch {
            Ok(message_batch) => {
                i = 1;
                message_batch
            }
            Err(e) => {
                error!("get_message from queue {} failed: {:?}", queue_url, e);
                tokio::time::delay_for(std::time::Duration::from_millis(i * 250)).await;
                i += 1;
                continue;
            }
        };
        let message_batch_len = message_batch.len();

        info!("Received {} messages", message_batch_len);

        if message_batch.is_empty() {
            tokio::time::delay_for(std::time::Duration::from_millis(250)).await;
            continue;
        }
        // We can't parallelize this because of the shared mutable state
        // of the retriever, emitter, and serializer

        // We could just preallocate multiple of each - like 10 of each,
        // and then just pick one of them at a time.

        // let mut tasks = Vec::with_capacity(message_batch.len());
        let combos = message_batch
            .into_iter()
            .zip(&mut *event_handler)
            .zip(&mut *s3_payload_retriever)
            .zip(&mut *s3_emitter)
            .zip(&mut *serializer)
            .zip(&mut *cache);

        let mut process_futs = Vec::with_capacity(message_batch_len);
        for combo in combos {
            let (
                (((((next_message, event_handler), s3_payload_retriever)), s3_emitter), serializer),
                cache,
            ) = combo;
            let p = process_message(
                next_message,
                queue_url.clone(),
                dead_letter_queue_url.clone(),
                cache,
                sqs_client.clone(),
                event_handler,
                s3_payload_retriever,
                s3_emitter,
                serializer,
                metric_reporter.clone(),
            );
            process_futs.push(p);
        }
        futures::future::join_all(process_futs).await;
    }
}

pub async fn process_loop<
    CacheT,
    SInit,
    SqsT,
    DecoderT,
    DecoderErrorT,
    EventHandlerT,
    InputEventT,
    OutputEventT,
    HandlerErrorT,
    SerializerErrorT,
    S3ClientT,
    OnEmit,
    OnEmitError,
    F,
    CompletionEventSerializerT,
>(
    queue_url: String,
    dead_letter_queue_url: String,
    cache: &mut [CacheT; 10],
    sqs_client: SqsT,
    event_handler: &mut [EventHandlerT; 10],
    s3_payload_retriever: &mut [S3PayloadRetriever<S3ClientT, SInit, DecoderT, InputEventT, DecoderErrorT>;
             10],
    s3_emitter: &mut [S3EventEmitter<S3ClientT, F, OnEmit, OnEmitError>; 10],
    serializer: &mut [CompletionEventSerializerT; 10],
    mut metric_reporter: MetricReporter<Stdout>,
) where
    CacheT: crate::cache::Cache + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S3ClientT) + Clone + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
    DecoderT: PayloadDecoder<InputEventT, DecoderError = DecoderErrorT> + Clone + Send + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
    InputEventT: Send,
    EventHandlerT:
        EventHandler<InputEvent = InputEventT, OutputEvent = OutputEventT, Error = HandlerErrorT>,
    OutputEventT: Clone + Send + Sync + 'static,
    HandlerErrorT: CheckedError + Debug + Send + Sync + 'static,
    SerializerErrorT: Error + Debug + Send + Sync + 'static,
    S3ClientT: S3 + Clone + Send + Sync + 'static,
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
    OnEmit: Clone + OnEventEmit<Error = OnEmitError> + Send + Sync + 'static,
    OnEmitError: CheckedError + Send,
    CompletionEventSerializerT: CompletionEventSerializer<
        CompletedEvent = OutputEventT,
        Output = Vec<u8>,
        Error = SerializerErrorT,
    >,
{
    loop {
        debug!("Outer process loop");
        let f = _process_loop(
            queue_url.clone(),
            dead_letter_queue_url.clone(),
            cache,
            sqs_client.clone(),
            event_handler,
            s3_payload_retriever,
            s3_emitter,
            serializer,
            metric_reporter.clone(),
        );
        let f = AssertUnwindSafe(f);
        if let Err(e) = f.catch_unwind().await {
            if let Some(e) = e.downcast_ref::<Box<dyn std::fmt::Debug>>() {
                error!("Processing loop failed {:?}", e);
            } else {
                error!("Unexpected error");
            }
            tokio::time::delay_for(std::time::Duration::from_millis(200)).await;
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
