#![type_length_limit = "1334469"]

use graph_generator_lib::run_graph_generator;
pub use grapl_service::{decoder::{ZstdDecoder,
                                  ZstdDecoderError},
                        serialization::{GraphDescriptionSerializer,
                                        GraphDescriptionSerializerError}};
use log::*;

use crate::{generator::SysmonSubgraphGenerator,
            metrics::SysmonSubgraphGeneratorMetrics};

mod generator;
mod metrics;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();
    let service_name = env.service_name.clone();
    info!("Starting sysmon-subgraph-generator");
    run_graph_generator(
        env,
        move |cache| {
            SysmonSubgraphGenerator::new(cache, SysmonSubgraphGeneratorMetrics::new(&service_name))
        },
        ZstdDecoder::default(),
    )
    .await;

    Ok(())
}
