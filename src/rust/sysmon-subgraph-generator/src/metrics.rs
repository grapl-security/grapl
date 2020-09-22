use failure::Error;
use grapl_observe::metric_reporter::{common_strs, MetricReporter, TagPair};
use log::*;
use std::io::Stdout;

#[derive(Clone)]
pub struct SysmonSubgraphGeneratorMetrics {
    metric_reporter: MetricReporter<Stdout>,
}

impl SysmonSubgraphGeneratorMetrics {
    pub fn new() -> SysmonSubgraphGeneratorMetrics {
        SysmonSubgraphGeneratorMetrics {
            metric_reporter: MetricReporter::<Stdout>::new(),
        }
    }
}

impl SysmonSubgraphGeneratorMetrics {
    pub fn report_handle_event_success(&mut self, failed: &Option<Error>) {
        let reported_status = if let Some(_) = failed {
            common_strs::FAIL
        } else {
            common_strs::SUCCESS
        };
        self.metric_reporter
            .gauge(
                "sysmon-generator-completion",
                1.0,
                &[TagPair(common_strs::STATUS, reported_status)],
            )
            .unwrap_or_else(|e| warn!("Metric failed: {}", e))
    }
}
