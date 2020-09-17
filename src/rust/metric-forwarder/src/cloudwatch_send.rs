use crate::cloudwatch_logs_parse::Stat;
use crate::error::MetricForwarderError;
use async_trait::async_trait;
use futures::future;
use log::info;
use log::warn;
use rayon::prelude::*;
use rusoto_cloudwatch::PutMetricDataError;
use rusoto_cloudwatch::{CloudWatch, MetricDatum, PutMetricDataInput};
use rusoto_core::RusotoError;
use statsd_parser;
use statsd_parser::Metric;

mod units {
    // strings accepted by CloudWatch MetricDatum.unit
    pub const COUNT: &'static str = "Count";
    pub const MILLIS: &'static str = "Milliseconds";
}

type PutResult = Result<(), RusotoError<PutMetricDataError>>;

#[async_trait]
pub trait CloudWatchPutter {
    // a subset of trait CloudWatch with the 1 function we want
    async fn put_metric_data(&self, input: PutMetricDataInput) -> PutResult;
}

#[async_trait]
impl<T> CloudWatchPutter for T
where
    T: CloudWatch + Sync + Send,
{
    async fn put_metric_data(&self, input: PutMetricDataInput) -> PutResult {
        CloudWatch::put_metric_data(self, input).await
    }
}

// Hardcoded for now to defer making a decision on how we want to do this
const CLOUDWATCH_NAMESPACE: &'static str = "grapl";

pub async fn put_metric_data(
    client: &impl CloudWatchPutter,
    metrics: &[MetricDatum],
) -> Result<(), MetricForwarderError> {
    /*
    Call Cloudwatch to insert metric data. Does batching on our behalf.
    */

    let chunks = metrics.chunks(20).map(|chunk| chunk.to_vec());
    let put_requests = chunks.map(|data: Vec<MetricDatum>| PutMetricDataInput {
        // TODO: should we do different namespaces for different services?
        namespace: CLOUDWATCH_NAMESPACE.to_string(),
        metric_data: data,
    });

    let request_futures = put_requests.map(|input| client.put_metric_data(input));
    let responses: Vec<PutResult> = future::join_all(request_futures).await;

    // TODO: retries

    // bubble up 1 of N failures
    let num_failures = responses.iter().filter(|resp| resp.is_err()).count();
    info!(
        "Sent {} batch-requests to Cloudwatch, of which {} failed",
        responses.len(),
        num_failures
    );
    let first_failure = responses.iter().filter(|resp| resp.is_err()).next();
    match first_failure {
        Some(Err(e)) => Err(MetricForwarderError::PutMetricDataError(e.to_string())),
        _ => Ok(()),
    }
}

pub fn statsd_as_cloudwatch_metric_bulk(
    parsed_stats: Vec<Result<Stat, MetricForwarderError>>,
) -> Vec<MetricDatum> {
    /*
    Convert the platform-agnostic Stat type to Cloudwatch-specific type.
    */
    parsed_stats
        .par_iter()
        // You will note that we drop metrics we couldn't parse. Theoretically it should never happen, but would be nice to know.
        // TODO: self-instrumentation around "how many stats do we drop?" Theoretically 0. We warn in the mean time.
        .filter_map(|stat_res| match stat_res {
            Ok(stat) => Some(statsd_as_cloudwatch_metric(stat)),
            Err(e) => {
                warn!("Dropped metric: {}", e);
                None
            }
        })
        .collect()
}

impl From<Stat> for MetricDatum {
    fn from(s: Stat) -> MetricDatum {
        statsd_as_cloudwatch_metric(&s)
    }
}

fn statsd_as_cloudwatch_metric(stat: &Stat) -> MetricDatum {
    let (unit, value, _sample_rate) = match &stat.msg.metric {
        // Yes, gauge and counter are - for our purposes - basically both Count
        Metric::Gauge(g) => (units::COUNT, g.value, g.sample_rate),
        Metric::Counter(c) => (units::COUNT, c.value, c.sample_rate),
        Metric::Histogram(h) => (units::MILLIS, h.value, h.sample_rate),
        _ => panic!("How the heck did you get an unsupported metric type in here?"),
    };
    let datum = MetricDatum {
        metric_name: stat.msg.name.to_string(),
        timestamp: stat.timestamp.to_string().into(),
        unit: unit.to_string().into(),
        value: value.into(),
        // TODO dimensions == tags
        // TODO seems like cloudwatch has no concept of sample rate, lol
        // many of the following are useful for batching:
        // e.g. counts: [1, 5] + values: [1.0, 2.0] means that
        // 1.0 was observed 1x && 2.0 was observed 5x
        counts: None,
        values: None,
        dimensions: None,
        statistic_values: None,
        storage_resolution: None,
    };
    datum
}

#[cfg(test)]
mod tests {
    use crate::cloudwatch_logs_parse::Stat;
    use crate::cloudwatch_send::{
        put_metric_data, statsd_as_cloudwatch_metric, units, CloudWatchPutter, PutResult,
    };
    use crate::error::MetricForwarderError;
    use async_trait::async_trait;
    use rusoto_cloudwatch::PutMetricDataError;
    use rusoto_cloudwatch::{MetricDatum, PutMetricDataInput};
    use rusoto_core::RusotoError;
    use statsd_parser;

    #[test]
    fn test_convert_one_datum() {
        let ts = "im timestamp".to_string();
        let name = "im a name".to_string();
        let counter = statsd_parser::Counter {
            value: 12.3,
            sample_rate: Some(0.5),
        };
        let stat = Stat {
            timestamp: ts.clone(),
            msg: statsd_parser::Message {
                name: name.clone(),
                tags: None,
                metric: statsd_parser::Metric::Counter(counter),
            },
        };

        let datum: MetricDatum = statsd_as_cloudwatch_metric(&stat);
        assert_eq!(&datum.metric_name, &name);
        assert_eq!(&datum.timestamp.expect(""), &ts);
        assert_eq!(datum.value.expect(""), 12.3);
        assert_eq!(datum.unit.expect(""), units::COUNT);
    }

    pub struct MockCloudwatchClient {
        response_fn: fn(PutMetricDataInput) -> PutResult,
    }
    impl MockCloudwatchClient {
        fn return_ok(_input: PutMetricDataInput) -> PutResult {
            Ok(())
        }

        fn return_an_err(_input: PutMetricDataInput) -> PutResult {
            Err(RusotoError::Service(
                PutMetricDataError::InternalServiceFault("ya goofed".to_string()),
            ))
        }
    }

    #[async_trait]
    impl CloudWatchPutter for MockCloudwatchClient {
        async fn put_metric_data(&self, input: PutMetricDataInput) -> PutResult {
            return (self.response_fn)(input);
        }
    }

    fn some_stat() -> Stat {
        Stat {
            timestamp: "ts".into(),
            msg: statsd_parser::Message {
                name: "msg".into(),
                metric: statsd_parser::Metric::Counter(statsd_parser::Counter {
                    value: 123.45,
                    sample_rate: None,
                }),
                tags: None,
            },
        }
    }

    #[tokio::test]
    async fn test_put_metric_data_client_ok() {
        let cw_client = MockCloudwatchClient {
            response_fn: MockCloudwatchClient::return_ok,
        };
        let data = vec![some_stat().into(), some_stat().into()];
        let result = put_metric_data(&cw_client, &data).await;
        assert_eq!(result, Ok(()))
    }

    #[tokio::test]
    async fn test_put_metric_data_client_err() -> Result<(), ()> {
        let cw_client = MockCloudwatchClient {
            response_fn: MockCloudwatchClient::return_an_err,
        };
        let data = vec![some_stat().into(), some_stat().into()];
        let result = put_metric_data(&cw_client, &data).await;
        match result {
            Err(MetricForwarderError::PutMetricDataError(_)) => Ok(()),
            _ => Err(()),
        }
    }
}
