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
use chrono::Utc;
use grapl_config::*;
use grapl_observe::metric_reporter::MetricReporter;
use sqs_lambda::event_handler::Completion;
use sqs_lambda::sqs_completion_handler::CompletionPolicy;
use sqs_lambda::sqs_consumer::{ConsumePolicy, ConsumePolicyBuilder};
use std::io::Stdout;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();

    let metrics = SysmonSubgraphGeneratorMetrics::new(&env.service_name);

    if grapl_config::is_local() {
        info!("Starting sysmon-subgraph-generator locally");
        let generator = SysmonSubgraphGenerator::new(NopCache {}, metrics);

        run_graph_generator(
            generator,
            ZstdDecoder::default(),
            ConsumePolicyBuilder::default(),
            CompletionPolicy::new(
                1,                      // Buffer up to 1 message
                Duration::from_secs(1), // Buffer for up to 1 second
            ),
            MetricReporter::<Stdout>::new("sysmon-subgraph-generator"),
        )
        .await;
    } else {
        info!("Starting sysmon-subgraph-generator in aws");

        let generator = SysmonSubgraphGenerator::new(event_cache().await, metrics);

        let completion_policy = ConsumePolicyBuilder::default()
            .with_max_empty_receives(1)
            .with_stop_at(Duration::from_secs(10));

        run_graph_generator(
            generator,
            ZstdDecoder::default(),
            completion_policy,
            CompletionPolicy::new(10, Duration::from_secs(2)),
            MetricReporter::<Stdout>::new("sysmon-subgraph-generator"),
        )
        .await;
    }

    Ok(())
}
