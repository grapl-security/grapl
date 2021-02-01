use std::{error::Error,
          future::Future,
          io::Stdout,
          time::{Duration,
                 SystemTime,
                 UNIX_EPOCH}};

use grapl_observe::metric_reporter::MetricReporter;
use log::info;
use rusoto_s3::S3;
use rusoto_sqs::Sqs;
use tracing::debug;

use crate::{cache::Cache,
            completion_event_serializer::CompletionEventSerializer,
            event_decoder::PayloadDecoder,
            event_handler::EventHandler,
            event_processor::{EventProcessor,
                              EventProcessorActor},
            event_retriever::S3PayloadRetriever,
            local_sqs_service_options::{LocalSqsServiceOptions,
                                        LocalSqsServiceOptionsBuilder},
            s3_event_emitter::S3EventEmitter,
            sqs_completion_handler::{SqsCompletionHandler,
                                     SqsCompletionHandlerActor},
            sqs_consumer::{IntoDeadline,
                           SqsConsumer,
                           SqsConsumerActor}};

fn time_based_key_fn(_event: &[u8]) -> String {
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!("{}/{}-{}", cur_day, cur_ms, uuid::Uuid::new_v4())
}

#[tracing::instrument(skip(
    queue_url,
    dest_bucket,
    _deadline,
    s3_init,
    s3_client,
    sqs_client,
    event_decoder,
    event_encoder,
    event_handler,
    cache,
    metric_reporter,
    on_ack,
    on_emit,
    options
))]
pub async fn local_sqs_service_with_options<
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
    dest_bucket: impl Into<String>,
    _deadline: impl IntoDeadline,
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
    options: LocalSqsServiceOptions,
) -> Result<(), Box<dyn std::error::Error>>
where
    SInit: (Fn(String) -> S3T) + Clone + Send + Sync + 'static,
    S3T: S3 + Clone + Send + Sync + 'static,
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
    let queue_url = queue_url.into();
    let dest_bucket = dest_bucket.into();

    let (tx, shutdown_notify) = tokio::sync::oneshot::channel();

    let (sqs_completion_handler, sqs_completion_handle) =
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
            options.completion_policy,
            on_ack,
            cache,
            metric_reporter.clone(),
        ));

    debug!("Created SqsCompletionHandler");

    let (sqs_consumer, sqs_consumer_handle) = SqsConsumerActor::new(SqsConsumer::new(
        sqs_client.clone(),
        queue_url.clone(),
        options.consume_policy,
        sqs_completion_handler.clone(),
        metric_reporter.clone(),
        tx,
    ))
    .await;

    debug!("Created SqsConsumerActor");

    let event_processors: Vec<_> = (0..1)
        .into_iter()
        .map(|_| {
            EventProcessorActor::new(EventProcessor::new(
                sqs_consumer.clone(),
                sqs_completion_handler.clone(),
                event_handler.clone(),
                S3PayloadRetriever::new(
                    s3_init.clone(),
                    event_decoder.clone(),
                    metric_reporter.clone(),
                ),
                metric_reporter.clone(),
            ))
        })
        .collect();
    debug!("created event_processors");

    futures::future::join_all(event_processors.iter().map(|ep| ep.0.start_processing())).await;

    drop(event_processors);
    drop(sqs_consumer);
    drop(sqs_completion_handler);

    sqs_consumer_handle.await;
    sqs_completion_handle.await;

    shutdown_notify.await;

    info!("Delaying");
    tokio::time::delay_for(Duration::from_secs(15)).await;
    Ok(())
}

pub async fn local_sqs_service<
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
    dest_bucket: impl Into<String>,
    deadline: impl IntoDeadline,
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
    SInit: (Fn(String) -> S3T) + Clone + Send + Sync + 'static,
    S3T: S3 + Clone + Send + Sync + 'static,
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
    let options = LocalSqsServiceOptionsBuilder::default().build();
    local_sqs_service_with_options(
        queue_url,
        dest_bucket,
        deadline,
        s3_init,
        s3_client,
        sqs_client,
        event_decoder,
        event_encoder,
        event_handler,
        cache,
        metric_reporter,
        on_ack,
        on_emit,
        options,
    )
    .await
}
