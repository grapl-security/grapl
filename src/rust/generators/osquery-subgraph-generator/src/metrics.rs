use std::io::Stdout;

use grapl_observe::metric_reporter::{
    common_strs,
    MetricReporter,
    TagPair,
};

#[derive(Clone)]
pub struct OSQuerySubgraphGeneratorMetrics {
    metric_reporter: MetricReporter<Stdout>,
}

impl OSQuerySubgraphGeneratorMetrics {
    pub fn new(service_name: &str) -> OSQuerySubgraphGeneratorMetrics {
        OSQuerySubgraphGeneratorMetrics {
            metric_reporter: MetricReporter::<Stdout>::new(service_name),
        }
    }

    pub fn report_subgraph_generation(&mut self) {
        self.metric_reporter
            .gauge(
                "osquery-subgraph-generation",
                1.0,
                &[TagPair(common_strs::STATUS, common_strs::SUCCESS)],
            )
            .unwrap_or_else(|e| tracing::warn!(message="Metric failed.", error=?e))
    }
}
