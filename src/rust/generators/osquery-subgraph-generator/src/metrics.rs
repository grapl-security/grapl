use failure::Error;
use grapl_observe::metric_reporter::{common_strs, MetricReporter, TagPair};
use log::*;
use std::io::Stdout;

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
}

impl OSQuerySubgraphGeneratorMetrics {
    pub fn report_handle_event_success(&mut self, failed: &Option<&Error>) {
        let reported_status = if let Some(_) = failed {
            common_strs::FAIL
        } else {
            common_strs::SUCCESS
        };
        self.metric_reporter
            .gauge(
                "osquery-generator-completion",
                1.0,
                &[TagPair(common_strs::STATUS, reported_status)],
            )
            .unwrap_or_else(|e| warn!("Metric failed: {}", e))
    }
}
