use std::io::Stdout;

use grapl_observe::metric_reporter::{
    common_strs,
    MetricReporter,
    TagPair,
};

pub enum Status {
    Success,
    Partial,
    Failure,
}

impl Status {
    fn from_result<T, E>(r: &Result<T, Result<(T, E), E>>) -> Self {
        match r {
            Ok(_) => Status::Success,
            Err(Ok((_, _))) => Status::Partial,
            Err(Err(_)) => Status::Failure,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Status::Success => "success",
            Status::Partial => "partial",
            Status::Failure => "failure",
        }
    }
}

#[derive(Clone)]
pub struct SysmonGeneratorMetrics {
    metric_reporter: MetricReporter<Stdout>,
}

impl SysmonGeneratorMetrics {
    pub fn new(service_name: &str) -> SysmonGeneratorMetrics {
        SysmonGeneratorMetrics {
            metric_reporter: MetricReporter::<Stdout>::new(service_name),
        }
    }
}

impl SysmonGeneratorMetrics {
    #[tracing::instrument(skip(self, event_result))]
    pub fn report_handle_event_success<T, E>(
        &mut self,
        event_result: &Result<T, Result<(T, E), E>>,
    ) {
        let status = Status::from_result(event_result);
        self.metric_reporter
            .gauge(
                "sysmon-generator-completion",
                1.0,
                &[TagPair(common_strs::STATUS, status.to_str())],
            )
            .unwrap_or_else(|e| tracing::warn!(message="Metric failed.", error=?e))
    }

    #[tracing::instrument(skip(self, result))]
    pub fn report_subgraph_generation<T, E>(&mut self, result: &Result<T, E>) {
        let status = match result {
            Ok(_) => common_strs::SUCCESS,
            Err(_) => common_strs::FAIL,
        };

        self.metric_reporter
            .gauge(
                "sysmon-generation",
                1.0,
                &[TagPair(common_strs::STATUS, status)],
            )
            .unwrap_or_else(|e| tracing::warn!(message="Metric failed.", error=?e))
    }
}
