pub mod retriever;

use std::{error::Error,
          fmt::Debug,
          future::Future,
          io::Stdout,
          panic::AssertUnwindSafe,
          time::{Duration,
                 SystemTime,
                 UNIX_EPOCH}};

use event_emitter::EventEmitter;
use event_handler::EventHandler;
use event_retriever::S3PayloadRetriever;
use futures_util::FutureExt;
use grapl_observe::{metric_reporter::{tag,
                                      MetricReporter},
                    timers::TimedFutureExt};
use rusoto_s3::S3;
use rusoto_sqs::{ListQueuesError,
                 ListQueuesRequest,
                 Message as SqsMessage,
                 Sqs};
use s3_event_emitter::S3EventEmitter;
use tracing::{debug,
              error,
              info};

use crate::{cache::{Cache,
                    CacheResponse},
            completion_event_serializer::CompletionEventSerializer,
            errors::{CheckedError,
                     Recoverable},
            event_decoder::PayloadDecoder,
            event_handler::CompletedEvents,
            event_retriever::PayloadRetriever,
            event_status::EventStatus,
            s3_event_emitter::OnEventEmit};

pub mod cache;
pub mod completion_event_serializer;
pub mod errors;
pub mod event_decoder;
pub mod event_emitter;
pub mod event_handler;
pub use retriever::{event_retriever,
                    s3_event_retriever};
use rusoto_core::RusotoError;

use crate::sqs_timeout_manager::keep_alive;

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
    CacheT: Cache + Send,
{
    //todo: lift, or avoid this entirely
    let mut to_cache = Vec::with_capacity(completed.len());

    for (identity, status) in completed.identities.drain(..) {
        match status {
            EventStatus::Success | EventStatus::Failure => {
                to_cache.push(identity);
            }
            _ => (),
        }
    }

    cache.store_all(to_cache).await.unwrap_or_else(|e| {
        error!(
            error = e.to_string().as_str(),
            "Failed to store_all in cache"
        )
    });
}

#[tracing::instrument(skip(
    next_message,
    queue_url,
    dead_letter_queue_url,
    cache,
    sqs_client,
    event_handler,
    s3_payload_retriever,
    s3_emitter,
    serializer,
    metric_reporter,
))]
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
    let message_id = next_message.message_id.as_ref().unwrap().as_str();
    let inner_loop_span = tracing::span!(
        tracing::Level::INFO,
        "inner_loop_span",
        message_id = message_id,
    );
    let _enter = inner_loop_span.enter();

    if let Ok(CacheResponse::Hit) = cache.get(message_id.as_bytes().to_owned()).await {
        info!(
            message_id = message_id,
            "Message has already been processed",
        );
        rusoto_helpers::delete_message(
            sqs_client.clone(),
            queue_url.to_owned(),
            next_message.receipt_handle.expect("missing receipt_handle"),
            metric_reporter,
        )
        .await
        .unwrap_or_else(|e| error!("delete_message failed: {:?}", e));
        return;
    }
    info!(message_id = message_id, "Retrieving payload from",);
    let receipt_handle = next_message
        .receipt_handle
        .as_ref()
        .expect("missing receipt_handle")
        .to_owned();
    // Maintain an invisibility timeout for the message until we're done
    let msg_handle = keep_alive(
        sqs_client.clone(),
        receipt_handle.clone(),
        message_id.to_owned(),
        queue_url.clone(),
        30,
    );
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
            error!(
                queue_url = queue_url.as_str(),
                message_id = message_id,
                error = e.to_string().as_str(),
                "Failed to retrieve payload with"
            );
            if let Recoverable::Persistent = e.error_type() {
                rusoto_helpers::move_to_dead_letter(
                    sqs_client.clone(),
                    next_message.body.as_ref().unwrap(),
                    dead_letter_queue_url,
                    queue_url.to_owned(),
                    receipt_handle,
                    metric_reporter.clone(),
                )
                .await
                .unwrap_or_else(|e| error!("move_to_dead_letter failed: {:?}", e));
            }
            return;
        }
    };

    // todo: We can lift this
    let mut completed = CompletedEvents::default();

    let processing_result = async {
        let (processing_result, ms) = event_handler
            .handle_event(events, &mut completed)
            .timed()
            .await;
        metric_reporter
            .histogram(
                "event_handler.handle_event",
                ms as f64,
                &[tag("success", processing_result.is_ok())],
            )
            .unwrap_or_else(|e| error!("failed to report event_handler.handle_event.ms: {:?}", e));
        processing_result
    }
    .await;

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

            cache
                .store(next_message.message_id.clone().unwrap().into_bytes())
                .await
                .unwrap_or_else(|e| error!("cache.store failed: {:?}", e));
            cache_completed(cache, &mut completed).await;
            // ack the message - we could probably not block on this

            msg_handle.stop();
            rusoto_helpers::delete_message(
                sqs_client.clone(),
                queue_url.to_owned(),
                receipt_handle,
                metric_reporter.clone(),
            )
            .await
            .unwrap_or_else(|e| error!("delete_message failed: {:?}", e));
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
                .await
                .unwrap_or_else(|e| error!("move_to_dead_letter failed: {:?}", e));
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
                .await
                .unwrap_or_else(|e| error!("move_to_dead_letter failed: {:?}", e));
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

        let span = tracing::span!(
            tracing::Level::DEBUG,
            "inner_process_loop",
            queue_url = queue_url.as_str(),
        );
        let _enter = span.enter();
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
                error!(
                    queue_url = queue_url.as_str(),
                    error = e.to_string().as_str(),
                    "Failed to get_message from queue"
                );
                tokio::time::sleep(std::time::Duration::from_millis(i * 250)).await;
                i += 1;
                continue;
            }
        };
        let message_batch_len = message_batch.len();

        info!(message_batch_len = message_batch_len, "Received messages");

        if message_batch.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            continue;
        }
        let combos = message_batch
            .into_iter()
            .zip(&mut *event_handler)
            .zip(&mut *s3_payload_retriever)
            .zip(&mut *s3_emitter)
            .zip(&mut *serializer)
            .zip(&mut *cache);

        let mut process_futs = Vec::with_capacity(10);

        for combo in combos {
            let (
                ((((next_message, event_handler), s3_payload_retriever), s3_emitter), serializer),
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

        let all_processing = tokio::time::timeout(
            Duration::from_secs(30 * 15),
            futures::future::join_all(process_futs).timed(),
        );
        match all_processing.await {
            Ok((_r, ms)) => {
                metric_reporter
                    .histogram("sqs_executor.all_processing.ms", ms as f64, &[])
                    .unwrap_or_else(|e| {
                        error!("failed to report sqs_executor.all_processing.ms: {:?}", e)
                    });
            }
            Err(e) => error!("Timed out when processing messages: {:?}", e),
        };
    }
}

#[tracing::instrument(skip(
    cache,
    sqs_client,
    event_handler,
    s3_payload_retriever,
    s3_emitter,
    serializer,
    metric_reporter,
))]
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
    metric_reporter: MetricReporter<Stdout>,
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
        tracing::trace!("Outer process loop");
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
            if let Some(e) = e.downcast_ref::<Box<dyn std::error::Error + 'static>>() {
                error!(
                    queue_url = queue_url.as_str(),
                    error = e.to_string().as_str(),
                    "Processing loop panicked"
                );
            } else if let Some(e) = e.downcast_ref::<Box<dyn std::fmt::Debug>>() {
                error!(
                    error = format!("{:?}", e).as_str(),
                    "Processing loop panicked"
                );
            } else {
                error!("Unexpected error");
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }
}

async fn wait_loop<F, T, E>(max_tries: u32, f: impl Fn() -> F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    T: Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut errs = None;
    for _ in 0..max_tries {
        match (f)().await {
            Ok(t) => return Ok(t),
            Err(e) => {
                errs = Some(Err(e));
            }
        };

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    if let Some(e) = errs {
        e
    } else {
        unreachable!()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WaitForSqsError {
    #[error("ListQueuesError")]
    ListQueuesError(#[from] RusotoError<ListQueuesError>),
    #[error("EmptyList")]
    EmptyList,
}

pub async fn wait_for_sqs(
    sqs_client: impl Sqs,
    queue_name_prefix: impl Into<String>,
) -> Result<(), WaitForSqsError> {
    let queue_name_prefix = queue_name_prefix.into();
    wait_loop(150, || async {
        let list_res = sqs_client
            .list_queues(ListQueuesRequest {
                max_results: None,
                next_token: None,
                queue_name_prefix: Some(queue_name_prefix.clone()),
            })
            .await;
        match list_res {
            Err(e) => {
                debug!("Waiting for sqs to become available: {:?}", e);
                Err(WaitForSqsError::ListQueuesError(e))
            }
            Ok(res) => {
                if let Some(res) = res.queue_urls {
                    if res.is_empty() {
                        return Err(WaitForSqsError::EmptyList);
                    }
                }
                Ok(())
            }
        }
    })
    .await?;

    Ok(())
}

pub fn time_based_key_fn(_event: &[u8]) -> String {
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!("{}/{}-{}", cur_day, cur_ms, uuid::Uuid::new_v4())
}
