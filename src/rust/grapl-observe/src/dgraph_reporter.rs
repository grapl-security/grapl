use std::time::Duration;

use crate::{
    metric_error::MetricError,
    metric_reporter::{MetricReporter, TagPair},
};

pub trait DgraphMetricReporter<T: std::io::Write> {
    fn mutation(
        &mut self,
        mutation_response: &dgraph_tonic::MutationResponse,
        tags: &[TagPair],
    ) -> Result<(), MetricError>;
    fn query(
        &mut self,
        query_response: &dgraph_tonic::Response,
        tags: &[TagPair],
    ) -> Result<(), MetricError>;
}

impl<T: std::io::Write> DgraphMetricReporter<T> for MetricReporter<T> {
    fn mutation(
        &mut self,
        mutation_response: &dgraph_tonic::MutationResponse,
        tags: &[TagPair],
    ) -> Result<(), MetricError> {
        let latency = mutation_response
            .latency
            .as_ref()
            .expect("Latency missing from mut response");
        let total_ms = Duration::from_nanos(latency.total_ns).as_millis();
        self.histogram("dgraph_mutation.total_ms", total_ms as f64, tags)
    }

    fn query(
        &mut self,
        query_response: &dgraph_tonic::Response,
        tags: &[TagPair],
    ) -> Result<(), MetricError> {
        let latency = query_response
            .latency
            .as_ref()
            .expect("Latency missing from query response");
        let total_ms = Duration::from_nanos(latency.total_ns).as_millis();
        self.histogram("dgraph_query.total_ms", total_ms as f64, tags)
    }
}
