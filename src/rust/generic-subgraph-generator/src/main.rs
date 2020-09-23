#![type_length_limit = "1232619"]

mod generator;
mod models;
mod serialization;
mod tests;

use sqs_lambda::cache::{Cache, CacheResponse, Cacheable, NopCache};

use tracing::*;

use graph_generator_lib::run_graph_generator;
use grapl_config::event_cache;

use crate::generator::GenericSubgraphGenerator;
use crate::serialization::ZstdJsonDecoder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    grapl_config::init_grapl_env!();

    info!("Starting generic-subgraph-generator");

    if grapl_config::is_local() {
        let generator = GenericSubgraphGenerator::new(NopCache {});

        run_graph_generator(generator, ZstdJsonDecoder::default()).await;
    } else {
        let generator = GenericSubgraphGenerator::new(event_cache().await);

        run_graph_generator(generator, ZstdJsonDecoder::default()).await;
    }

    Ok(())
}
