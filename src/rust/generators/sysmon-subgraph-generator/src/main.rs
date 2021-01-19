#![type_length_limit = "1334469"]

use log::*;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;

use graph_generator_lib::run_graph_generator;
use grapl_config::env_helpers::FromEnv;
use grapl_config::*;
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::s3_event_emitter::S3ToSqsEventNotifier;
use sqs_executor::{make_ten, time_based_key_fn};

use crate::generator::SysmonSubgraphGenerator;
use crate::metrics::SysmonSubgraphGeneratorMetrics;

pub use grapl_service::decoder::{ZstdDecoder, ZstdDecoderError};
pub use grapl_service::serialization::{SubgraphSerializer, SubgraphSerializerError};

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
