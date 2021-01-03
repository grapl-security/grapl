#![type_length_limit = "1334469"]

use log::*;

use rusoto_sqs::SqsClient;

use grapl_config::*;
use grapl_observe::metric_reporter::MetricReporter;

use sqs_executor::event_retriever::S3PayloadRetriever;

use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::{make_ten, time_based_key_fn};

use crate::generator::SysmonSubgraphGenerator;
use crate::metrics::SysmonSubgraphGeneratorMetrics;
use crate::serialization::{SubgraphSerializer, ZstdDecoder};
use grapl_config::env_helpers::FromEnv;
use rusoto_s3::S3Client;
use sqs_executor::s3_event_emitter::S3ToSqsEventNotifier;
use graph_generator_lib::run_graph_generator;

mod generator;
mod metrics;
mod models;
mod serialization;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    let service_name = env.service_name.clone();
    info!("Starting sysmon-subgraph-generator");
    run_graph_generator(
        env,
        move |cache| SysmonSubgraphGenerator::new(
            cache,
            SysmonSubgraphGeneratorMetrics::new(&service_name),
        ),
        ZstdDecoder::default(),
    ).await;

    Ok(())
}
