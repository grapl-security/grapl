#![type_length_limit = "1334469"]

mod generator;
mod metrics;
mod models;
mod serialization;

use sqs_lambda::cache::NopCache;

use graph_generator_lib::*;

use log::*;

use crate::generator::SysmonSubgraphGenerator;
use crate::metrics::SysmonSubgraphGeneratorMetrics;
use crate::serialization::ZstdDecoder;
use grapl_config::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    info!("Starting sysmon-subgraph-generator");

    let metrics = SysmonSubgraphGeneratorMetrics::new(&env.service_name);

    if grapl_config::is_local() {
        let generator = SysmonSubgraphGenerator::new(NopCache {}, metrics);

        run_graph_generator(generator, ZstdDecoder::default()).await;
    } else {
        let generator = SysmonSubgraphGenerator::new(event_cache().await, metrics);

        run_graph_generator(generator, ZstdDecoder::default()).await;
    }

    Ok(())
}
