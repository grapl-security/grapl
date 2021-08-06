use std::collections::BTreeMap;

use async_trait::async_trait;
use futures::future;
use log::{
    info,
    warn,
};
use rayon::prelude::*;
use rusoto_cloudwatch::{
    CloudWatch,
    Dimension,
    MetricDatum,
    PutMetricDataError,
    PutMetricDataInput,
};
use rusoto_core::RusotoError;
use statsd_parser::{
    self,
    Metric,
};

use crate::{
    cloudwatch_logs_parse::Stat,
    error::MetricForwarderError,
};

pub mod cw_units {
    // strings accepted by CloudWatch MetricDatum.unit
    pub const COUNT: &str = "Count";
    pub const MILLIS: &str = "Milliseconds";
    pub const MICROS: &str = "Microseconds";
    pub const SECONDS: &str = "Seconds";
}
const RESERVED_UNIT_TAG: &str = "_unit";

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

pub async fn put_metric_data(
    client: &impl CloudWatchPutter,
    metrics: &[MetricDatum],
    namespace: &str,
) -> Result<(), MetricForwarderError> {
    /*
    Call Cloudwatch to insert metric data. Does batching on our behalf.
    */
    let chunks = metrics.chunks(20).map(|chunk| chunk.to_vec());
    let put_requests = chunks.map(|data: Vec<MetricDatum>| PutMetricDataInput {
        namespace: namespace.to_string(),
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

/// A nice simplification of our Metric Forwarder is that:
///  since we process things per-log-group,
/// and one log group only has things from one service
/// ---> each execution of this lambda will only be dealing with 1 namespace.
/// We simply assert this invariant in this function.
/// A more robust way would be to group by namespace, but, eh, not needed for us
pub fn get_namespace(stats: &[Stat]) -> Result<String, MetricForwarderError> {
    if let Some(first) = stats.get(0) {
        let expected_namespace = first.service_name.clone();
        let find_different_namespace = stats
            .par_iter()
            .find_any(|s| s.service_name != expected_namespace);
        match find_different_namespace {
            Some(different) => Err(MetricForwarderError::MoreThanOneNamespaceError(
                expected_namespace.to_string(),
                different.service_name.to_string(),
            )),
            None => Ok(expected_namespace),
        }
    } else {
        // I don't expect this to ever happen.
        Err(MetricForwarderError::NoLogsError())
    }
}

pub fn filter_invalid_stats(parsed_stats: Vec<Result<Stat, MetricForwarderError>>) -> Vec<Stat> {
    parsed_stats
        .into_par_iter()
        .filter_map(|stat_res| match stat_res {
            Ok(stat) => Some(stat),
            Err(e) => {
                warn!("Dropped metric: {}", e);
                None
            }
        })
        .collect()
}

pub fn statsd_as_cloudwatch_metric_bulk(parsed_stats: Vec<Stat>) -> Vec<MetricDatum> {
    /*
    Convert the platform-agnostic Stat type to Cloudwatch-specific type.
    */
    parsed_stats
        .into_par_iter()
        .map(statsd_as_cloudwatch_metric)
        .collect()
}

impl From<Stat> for MetricDatum {
    fn from(s: Stat) -> MetricDatum {
        statsd_as_cloudwatch_metric(s)
    }
}

#[derive(Default)]
struct Dimensions(Vec<Dimension>);
/// create Dimensions from statsd Message.tags
impl From<&BTreeMap<String, String>> for Dimensions {
    fn from(source: &BTreeMap<String, String>) -> Dimensions {
        Dimensions(
            source
                .into_iter()
                .map(|(k, v)| Dimension {
                    name: k.to_string(),
                    value: v.to_string(),
                })
                .collect(),
        )
    }
}

impl Dimensions {
    fn find_dimension(&self, dimension_name: &str) -> Option<Dimension> {
        let found = self.0.iter().find(|d| d.name == dimension_name);
        return found.map(Dimension::clone);
    }

    fn remove_dimension(&mut self, dimension: &Dimension) {
        self.0.retain(|d| d != dimension)
    }
}

fn override_unit_from_dims(mut unit: &'static str, dims: &mut Dimensions) -> &'static str {
    // https://github.com/grapl-security/issue-tracker/issues/132
    // Read the optional `_unit` dimension and use it to override the default
    // assumption of milliseconds.
    // Remove the _unit dimension from dimensions.

    let units_dimension_option = dims.find_dimension(RESERVED_UNIT_TAG);
    if let Some(units_dimension) = units_dimension_option {
        // Right now, we only specify `_unit` for histograms.
        assert_eq!(unit, cw_units::MILLIS);

        unit = match &units_dimension.value[..] {
            "millis" => cw_units::MILLIS,
            "micros" => cw_units::MICROS,
            "seconds" => cw_units::SECONDS,
            _ => {
                warn!("Unexpected unit: {}", units_dimension.value);
                unit
            }
        };
        dims.remove_dimension(&units_dimension);
    }
    unit
}

fn statsd_as_cloudwatch_metric(stat: Stat) -> MetricDatum {
    let (mut unit, value, _sample_rate) = match stat.msg.metric {
        // Yes, gauge and counter are - for our purposes - basically both Count
        Metric::Gauge(g) => (cw_units::COUNT, g.value, g.sample_rate),
        Metric::Counter(c) => (cw_units::COUNT, c.value, c.sample_rate),
        Metric::Histogram(h) => (cw_units::MILLIS, h.value, h.sample_rate),
        _ => panic!("How the heck did you get an unsupported metric type in here?"),
    };

    let mut dims: Dimensions = stat
        .msg
        .tags
        .as_ref()
        .map(|tags| tags.into())
        .unwrap_or_default();

    unit = override_unit_from_dims(unit, &mut dims);

    let Dimensions(dims_vec) = dims;
    // AWS doesn't like sending it an empty list
    let dims_option = match dims_vec.is_empty() {
        true => None,
        false => Some(dims_vec),
    };

    let datum = MetricDatum {
        metric_name: stat.msg.name.to_string(),
        timestamp: stat.timestamp.to_string().into(),
        unit: unit.to_string().into(),
        value: value.into(),
        // TODO seems like cloudwatch has no concept of sample rate, lol
        // many of the following are useful for batching:
        // e.g. counts: [1, 5] + values: [1.0, 2.0] means that
        // 1.0 was observed 1x && 2.0 was observed 5x
        counts: None,
        values: None,
        dimensions: dims_option,
        statistic_values: None,
        storage_resolution: None,
    };
    datum
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVICE_NAME: &'static str = "cool_service";

    #[test]
    fn test_convert_one_stat_into_datum() {
        let ts = "im timestamp".to_string();
        let name = "im a name".to_string();
        let service_name = SERVICE_NAME.into();
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
            service_name: service_name,
        };

        let datum: MetricDatum = stat.into();
        assert_eq!(&datum.metric_name, &name);
        assert_eq!(&datum.timestamp.expect(""), &ts);
        assert_eq!(datum.value.expect(""), 12.3);
        assert_eq!(datum.unit.expect(""), cw_units::COUNT);
        assert_eq!(datum.dimensions, None);
    }

    #[test]
    fn test_convert_one_stat_with_tags_into_datum() {
        let stat = some_stat_with_tags();
        let datum: MetricDatum = stat.into();
        assert_eq!(
            datum.dimensions.expect(""),
            vec![
                Dimension {
                    name: "tag1".into(),
                    value: "val1".into()
                },
                Dimension {
                    name: "tag2".into(),
                    value: "val2".into()
                },
            ]
        );
    }

    #[test]
    fn test_convert_one_stat_with_unit_tags_into_datum() {
        // Test the behavior specified in https://github.com/grapl-security/issue-tracker/issues/132
        let stat = some_histogram_stat_with_unit_tag();
        let datum: MetricDatum = stat.into();
        assert_eq!(
            &datum.dimensions.expect(""),
            &vec![
                Dimension {
                    name: "tag1".into(),
                    value: "val1".into()
                },
                // Resulting vec has no "_unit"
            ]
        );
        assert_eq!(&datum.unit.expect(""), &cw_units::MICROS);
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
            service_name: SERVICE_NAME.into(),
        }
    }

    fn some_stat_with_different_service_name() -> Stat {
        let mut stat = some_stat();
        stat.service_name = "another_service".to_string();
        stat
    }

    fn some_stat_with_tags() -> Stat {
        let mut tags = BTreeMap::<String, String>::new();
        tags.insert("tag2".into(), "val2".into());
        tags.insert("tag1".into(), "val1".into());
        // note - .iter() sorts these by key, so tag1 will show up first!
        Stat {
            timestamp: "ts".into(),
            msg: statsd_parser::Message {
                name: "msg".into(),
                metric: statsd_parser::Metric::Counter(statsd_parser::Counter {
                    value: 123.45,
                    sample_rate: None,
                }),
                tags: tags.into(),
            },
            service_name: SERVICE_NAME.into(),
        }
    }

    fn some_histogram_stat_with_unit_tag() -> Stat {
        let mut tags = BTreeMap::<String, String>::new();
        tags.insert("tag1".into(), "val1".into());
        tags.insert(RESERVED_UNIT_TAG.into(), "micros".into());
        Stat {
            timestamp: "ts".into(),
            msg: statsd_parser::Message {
                name: "msg".into(),
                metric: statsd_parser::Metric::Histogram(statsd_parser::Histogram {
                    value: 123.45,
                    sample_rate: None,
                }),
                tags: tags.into(),
            },
            service_name: SERVICE_NAME.into(),
        }
    }

    #[test]
    fn test_get_namespace_different_service_names() {
        let stats = vec![some_stat(), some_stat_with_different_service_name()];

        let result = get_namespace(&stats);
        match result {
            Ok(_) => panic!("shouldn't get anything here"),
            Err(e) => assert!(e
                .to_string()
                .contains("Expected cool_service, found another_service")),
        }
    }

    #[test]
    fn test_get_namespace_same_service_names() -> Result<(), MetricForwarderError> {
        let stats = vec![some_stat(), some_stat(), some_stat()];

        let result = get_namespace(&stats)?;
        assert_eq!(result, SERVICE_NAME);
        Ok(())
    }

    #[tokio::test]
    async fn test_put_metric_data_client_ok() {
        let cw_client = MockCloudwatchClient {
            response_fn: MockCloudwatchClient::return_ok,
        };
        let data = vec![some_stat().into(), some_stat().into()];
        let result = put_metric_data(&cw_client, &data, SERVICE_NAME).await;
        assert_eq!(result, Ok(()))
    }

    #[tokio::test]
    async fn test_put_metric_data_client_err() -> Result<(), ()> {
        let cw_client = MockCloudwatchClient {
            response_fn: MockCloudwatchClient::return_an_err,
        };
        let data = vec![some_stat().into(), some_stat().into()];
        let result = put_metric_data(&cw_client, &data, SERVICE_NAME).await;
        match result {
            Err(MetricForwarderError::PutMetricDataError(_)) => Ok(()),
            _ => Err(()),
        }
    }
}
