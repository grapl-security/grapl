pub use grapl_config;
pub use grapl_graph_descriptions::graph_description::*;
pub use grapl_observe::metric_reporter::MetricReporter;
use tracing::info;
use grapl_config::{ServiceEnv, event_caches};
use rusoto_sqs::SqsClient;
use rusoto_s3::S3Client;
use crate::grapl_config::env_helpers::{FromEnv, s3_event_emitters_from_env};
use sqs_executor::{make_ten, time_based_key_fn};
use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::event_decoder::PayloadDecoder;
use sqs_executor::event_handler::EventHandler;
use sqs_executor::s3_event_emitter::S3ToSqsEventNotifier;
use std::fmt::Debug;
use sqs_executor::errors::CheckedError;
use grapl_service::serialization::SubgraphSerializer;
use sqs_executor::redis_cache::RedisCache;

pub async fn run_graph_generator<
    InputEventT,
    HandlerErrorT,
    InitGenerator,
    PayloadDecoderT,
    DecoderErrorT,
    EventHandlerT,
>
(
    env: ServiceEnv,
    init_generator: InitGenerator,
    payload_decoder: PayloadDecoderT
)
    where
        InputEventT: Send + 'static,
        InitGenerator: Clone + Send + 'static + Fn(RedisCache) -> EventHandlerT,
        EventHandlerT: EventHandler<InputEvent = InputEventT, OutputEvent = Graph, Error = HandlerErrorT> + Send + Sync + 'static + Clone,
        HandlerErrorT: Debug + CheckedError + Send + Sync + 'static,
        PayloadDecoderT: PayloadDecoder<InputEventT, DecoderError=DecoderErrorT> + Send + Sync + Clone + 'static,
        DecoderErrorT: CheckedError + Send + 'static,
{
    let destination_bucket = grapl_config::dest_bucket();

    let sqs_client = SqsClient::from_env();
    let s3_client = S3Client::from_env();

    let cache = &mut event_caches(&env).await;

    let sysmon_subgraph_generator = &mut make_ten(async {
        (init_generator)(cache[0].clone())
    })
        .await;
    let serializer = &mut make_ten(async { SubgraphSerializer::default() }).await;

    let s3_emitter =
        &mut s3_event_emitters_from_env(&env, time_based_key_fn, S3ToSqsEventNotifier::from(&env))
            .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| {
                info!("Initializing new s3 client: {}", &region_str);
                grapl_config::env_helpers::init_s3_client(&region_str)
            },
            payload_decoder,
            MetricReporter::new(&env.service_name),
        )
    })
        .await;

    info!("Starting process_loop");
    let queue_url = grapl_config::source_queue_url();
    // queue urls are of the form
    // for example, http://sqs.us-east-1.amazonaws.com:9324/queue/grapl-sysmon-graph-generator-queue

    let queue_name = queue_url.split("/").last()
        .unwrap_or_else(|| panic!("invalid queue_url: {}", &queue_url));
    grapl_config::wait_for_sqs(SqsClient::from_env(), queue_name.clone()).await
        .unwrap_or_else(|e| panic!("never found queue: {} {}", queue_name, &queue_url));

    sqs_executor::process_loop(
        queue_url,
        std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL"),
        cache,
        sqs_client.clone(),
        sysmon_subgraph_generator,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
        .await;
}
//
// /// Graph generator implementations should invoke this function to begin processing new log events.
// ///
// /// ```rust,ignore
// /// #[tokio::main]
// /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
// ///     use sqs_lambda::cache::NopCache;
// ///     use graph_generator_lib::run_graph_generator;
// ///
// ///     grapl_config::init_grapl_env!();
// ///
// ///     run_graph_generator(
// ///         MyNewGenerator::new(),
// ///         MyDecoder::default()
// ///     ).await;
// ///
// ///     Ok(())
// /// }
// /// ```
// pub async fn run_graph_generator<
//     IE: Send + Sync + Clone + 'static,
//     EH: EventHandler<InputEvent = IE, OutputEvent = Graph, Error = sqs_lambda::error::Error>
//         + Send
//         + Sync
//         + Clone
//         + 'static,
//     ED: PayloadDecoder<IE> + Send + Sync + Clone + 'static,
// >(
//     generator: EH,
//     event_decoder: ED,
//     consume_policy: ConsumePolicyBuilder,
//     completion_policy: CompletionPolicy,
//     metric_reporter: MetricReporter<Stdout>,
// ) {
//     info!("IS_LOCAL={:?}", config::is_local());
//
//     if config::is_local() {
//         local::run_graph_generator_local(
//             generator,
//             event_decoder,
//             consume_policy,
//             completion_policy,
//             metric_reporter,
//         )
//         .await;
//     } else {
//         aws::run_graph_generator_aws(
//             generator,
//             event_decoder,
//             consume_policy,
//             completion_policy,
//             metric_reporter,
//         );
//     }
// }
