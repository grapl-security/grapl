#![type_length_limit = "1334469"]
#![feature(test)]
use graph_generator_lib::run_graph_generator;
pub use grapl_service::serialization::{GraphDescriptionSerializer,
                                       GraphDescriptionSerializerError};
use log::*;
use sysmon_subgraph_generator_lib::{generator::SysmonSubgraphGenerator,
                                    metrics::SysmonSubgraphGeneratorMetrics,
                                    serialization::SysmonDecoder};

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
        SysmonDecoder::default(),
    )
    .await;

    Ok(())
}
