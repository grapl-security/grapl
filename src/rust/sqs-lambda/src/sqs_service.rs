use std::error::Error;
use std::future::Future;
use std::io::Stdout;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::info;
use rusoto_s3::S3;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};

use grapl_observe::metric_reporter::MetricReporter;

use crate::cache::Cache;
use crate::completion_event_serializer::CompletionEventSerializer;
use crate::event_decoder::PayloadDecoder;
use crate::event_handler::EventHandler;
use crate::event_processor::{EventProcessor, EventProcessorActor};
use crate::event_retriever::S3PayloadRetriever;
use crate::s3_event_emitter::S3EventEmitter;
use crate::sqs_completion_handler::{
    CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor,
};
use crate::sqs_consumer::{ConsumePolicy, IntoDeadline, SqsConsumer, SqsConsumerActor};

fn time_based_key_fn(_event: &[u8]) -> String {
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!("{}/{}-{}", cur_day, cur_ms, uuid::Uuid::new_v4())
}

pub async fn sqs_service<
    S3T,
    SInit,
    SqsT,
    EventT,
    CompletedEventT,
    EventDecoderT,
    EventEncoderT,
    EventHandlerT,
    CacheT,
    OnAck,
    EmissionResult,
    OnEmission,
>(
    queue_url: impl Into<String>,
    initial_messages: impl IntoIterator<Item = rusoto_sqs::Message>,
    dest_bucket: impl Into<String>,
    consume_policy: ConsumePolicy,
    completion_policy: CompletionPolicy,
    s3_init: SInit,
    s3_client: S3T,
    sqs_client: SqsT,
    event_decoder: EventDecoderT,
    event_encoder: EventEncoderT,
    event_handler: EventHandlerT,
    cache: CacheT,
    metric_reporter: MetricReporter<Stdout>,
    on_ack: OnAck,
    on_emit: OnEmission,
) -> Result<(), Box<dyn std::error::Error>>
where
    S3T: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S3T) + Clone + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
    CompletedEventT: Clone + Send + Sync + 'static,
    EventT: Clone + Send + Sync + 'static,
    EventDecoderT: PayloadDecoder<EventT> + Clone + Send + Sync + 'static,
    EventEncoderT: CompletionEventSerializer<
            CompletedEvent = CompletedEventT,
            Output = Vec<u8>,
            Error = <EventHandlerT as EventHandler>::Error,
        > + Clone
        + Send
        + Sync
        + 'static,
    EventHandlerT: EventHandler<
            InputEvent = EventT,
            OutputEvent = CompletedEventT,
            Error = crate::error::Error,
        > + Clone
        + Send
        + Sync
        + 'static,
    CacheT: Cache + Clone + Send + Sync + 'static,
    OnAck: Fn(
            SqsCompletionHandlerActor<CompletedEventT, <EventHandlerT as EventHandler>::Error, SqsT>,
            Result<String, String>,
        ) + Send
        + Sync
        + 'static,
    OnEmission: Fn(String, String) -> EmissionResult + Send + Sync + 'static,
    EmissionResult:
        Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
{
    info!("sqs service init");
    let queue_url = queue_url.into();
    let dest_bucket = dest_bucket.into();

    let (tx, shutdown_notify) = tokio::sync::oneshot::channel();

    let (sqs_completion_handler, _sqs_completion_handle) =
        SqsCompletionHandlerActor::new(SqsCompletionHandler::new(
            sqs_client.clone(),
            queue_url.clone(),
            event_encoder,
            S3EventEmitter::new(
                s3_client.clone(),
                dest_bucket.clone(),
                time_based_key_fn,
                on_emit,
            ),
            completion_policy,
            on_ack,
            cache,
            metric_reporter.clone(),
        ));

    let (sqs_consumer, sqs_consumer_handle) = SqsConsumerActor::new(SqsConsumer::new(
        sqs_client.clone(),
        queue_url.clone(),
        consume_policy,
        sqs_completion_handler.clone(),
        metric_reporter.clone(),
        tx,
    ))
    .await;

    let event_processors: Vec<_> = (0..40)
        .into_iter()
        .map(|_| {
            EventProcessorActor::new(EventProcessor::new(
                sqs_consumer.clone(),
                sqs_completion_handler.clone(),
                event_handler.clone(),
                S3PayloadRetriever::new(s3_init.clone(), event_decoder.clone(), metric_reporter.clone()),
                metric_reporter.clone(),
            ))
        })
        .collect();

    futures::future::join_all(event_processors.iter().map(|ep| ep.0.start_processing())).await;

    let mut proc_iter = event_processors.iter().cycle();
    for message in initial_messages.into_iter() {
        let next_proc = proc_iter.next().unwrap();
        next_proc.0.process_event(message).await;
    }

    drop(event_processors);
    drop(sqs_consumer);
    drop(sqs_completion_handler);

    sqs_consumer_handle.await;
    shutdown_notify.await;
    Ok(())
}
