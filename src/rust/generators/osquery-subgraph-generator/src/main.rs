#![type_length_limit = "1232619"]
mod generator;
mod metrics;
mod parsers;
mod serialization;
mod tests;

use crate::generator::OSQuerySubgraphGenerator;
use crate::metrics::OSQuerySubgraphGeneratorMetrics;
use crate::serialization::OSQueryLogDecoder;
use graph_generator_lib::*;
use grapl_config::*;
use log::*;
use sqs_lambda::cache::NopCache;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    info!("Starting osquery-subgraph-generator");

    let metrics = OSQuerySubgraphGeneratorMetrics::new(&env.service_name);

    if env.is_local {
        let generator = OSQuerySubgraphGenerator::new(NopCache {}, metrics);

        run_graph_generator(generator, OSQueryLogDecoder::default()).await;
    } else {
        let generator = OSQuerySubgraphGenerator::new(event_cache().await, metrics);

        run_graph_generator(generator, OSQueryLogDecoder::default()).await;
    }

    Ok(())
}
