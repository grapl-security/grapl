#![type_length_limit = "1232619"]

mod generator;
mod metrics;
mod parsers;
mod serialization;
mod tests;

use std::{io::Stdout,
          time::Duration};

use graph_generator_lib::*;
use grapl_config::*;
use grapl_observe::metric_reporter::MetricReporter;
use log::*;
use sqs_lambda::{cache::NopCache,
                 sqs_completion_handler::CompletionPolicy,
                 sqs_consumer::ConsumePolicyBuilder};

use crate::{generator::OSQuerySubgraphGenerator,
            metrics::OSQuerySubgraphGeneratorMetrics,
            serialization::OSQueryLogDecoder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    info!("Starting osquery-subgraph-generator");

    let metrics = OSQuerySubgraphGeneratorMetrics::new(&env.service_name);

    if env.is_local {
        let generator = OSQuerySubgraphGenerator::new(NopCache {}, metrics);

        run_graph_generator(
            generator,
            OSQueryLogDecoder::default(),
            ConsumePolicyBuilder::default(),
            CompletionPolicy::new(
                1,                      // Buffer up to 1 message
                Duration::from_secs(1), // Buffer for up to 1 second
            ),
            MetricReporter::<Stdout>::new("osquery-subgraph-generator"),
        )
        .await;
    } else {
        let generator = OSQuerySubgraphGenerator::new(event_cache().await, metrics);
        let completion_policy = ConsumePolicyBuilder::default()
            .with_max_empty_receives(1)
            .with_stop_at(Duration::from_secs(10));

        run_graph_generator(
            generator,
            OSQueryLogDecoder::default(),
            completion_policy,
            CompletionPolicy::new(10, Duration::from_secs(2)),
            MetricReporter::<Stdout>::new("osquery-subgraph-generator"),
        )
        .await;
    }

    Ok(())
}
