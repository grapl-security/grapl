use std::io::Stdout;

use grapl_observe::metric_reporter::MetricReporter;

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
