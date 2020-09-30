mod aws;
mod local;
mod serialization;

use log::*;

use grapl_graph_descriptions::graph_description::*;
use grapl_config as config;

use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::EventHandler;

/// Graph generator implementations should invoke this function to begin processing new log events.
///
/// ```rust,ignore
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     use sqs_lambda::cache::NopCache;
///     use graph_generator_lib::run_graph_generator;
///
///     grapl_config::init_grapl_env!();
///
///     run_graph_generator(
///         MyNewGenerator::new(),
///         MyDecoder::default()
///     ).await;
///
///     Ok(())
/// }
/// ```
pub async fn run_graph_generator<
    IE: Send + Sync + Clone + 'static,
    EH: EventHandler<InputEvent = IE, OutputEvent = Graph, Error = sqs_lambda::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ED: PayloadDecoder<IE> + Send + Sync + Clone + 'static,
>(
    generator: EH,
    event_decoder: ED,
) {
    info!("IS_LOCAL={:?}", config::is_local());

    if config::is_local() {
        local::run_graph_generator_local(generator, event_decoder).await;
    } else {
        aws::run_graph_generator_aws(generator, event_decoder);
    }
}
