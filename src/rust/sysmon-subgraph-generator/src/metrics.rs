use failure::Error;
use grapl_observe::metric_reporter::{MetricReporter, TagPair};
use log::*;

#[derive(Clone)]
pub struct SysmonSubgraphGeneratorMetrics {
    pub metric_reporter: MetricReporter,
}

impl SysmonSubgraphGeneratorMetrics {
    pub fn report_handle_event_success(&mut self, failed: &Option<Error>) {
        let reported_status = if let Some(ref e) = failed {
            "failed"
        } else {
            "completed"
        };
        self.metric_reporter
            .gauge(
                "sysmon-generator-completion",
                1.0,
                &[TagPair("status", reported_status)],
            )
            .map_err(|e| warn!("Metric failed: {}", e));
    }
}
