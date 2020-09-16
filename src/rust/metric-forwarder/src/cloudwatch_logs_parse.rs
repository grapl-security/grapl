use aws_lambda_events::event::cloudwatch_logs::{CloudwatchLogsData, CloudwatchLogsLogEvent};
use rayon::prelude::*;
use statsd_parser;

use crate::error::MetricForwarderError;

#[derive(Debug)]
pub struct Stat {
    pub msg: statsd_parser::Message,
    pub timestamp: String,
}

const MONITORING_DELIM: &'static str = "|";

pub fn parse_logs(logs_data: CloudwatchLogsData) -> Vec<Result<Stat, MetricForwarderError>> {
    /*
    Parse the incoming, raw logs into a platform-agnostic Stat type.
    */
    logs_data
        .log_events
        .par_iter()
        .filter_map(|logs_log_event: &CloudwatchLogsLogEvent| logs_log_event.message.as_ref())
        .map(|s| parse_log(s))
        .collect()
}

fn parse_log(log_str: &str) -> Result<Stat, MetricForwarderError> {
    /*
    The input will look like
    MONITORING|timestamp|statsd|stuff|goes|here
     */
    let split: Vec<&str> = log_str.splitn(3, MONITORING_DELIM).collect();
    match &split[..] {
        [_monitoring, timestamp, statsd_component] => {
            let statsd_msg = statsd_parser::parse(statsd_component.to_string());
            statsd_msg
                .map(|msg| Stat {
                    timestamp: timestamp.to_string(),
                    msg: msg,
                })
                .map_err(|parse_err| {
                    MetricForwarderError::ParseStringToStatsdError(
                        parse_err.to_string(),
                        log_str.to_string(),
                    )
                })
        }
        _ => Err(MetricForwarderError::PoorlyFormattedLogLine(
            log_str.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::cloudwatch_logs_parse::Stat;
    use crate::cloudwatch_logs_parse::parse_log;
    use crate::cloudwatch_logs_parse::MONITORING_DELIM;
    use crate::error::MetricForwarderError;
    use statsd_parser::Counter;
    use statsd_parser::Gauge;
    use statsd_parser::Metric;

    fn expect_metric(input: &[&str], expected: Metric) -> Result<Stat, MetricForwarderError> {
        let input_joined = input.join(MONITORING_DELIM);
        let parsed = parse_log(input_joined.as_str())?;
        assert_eq!(parsed.msg.name, "some_\tstr");
        assert_eq!(
            parsed.msg.metric,
            expected,
        );
        Ok(parsed)
    }

    #[test]
    fn test_parse_one_log() -> Result<(), MetricForwarderError> {
        let input = [
            "MONITORING",
            "2017-04-26T10:41:09.023Z",
            // you'll note I threw a gross, extra \t in the metric name as an edge case
            "some_\tstr:12345.6|c|#some_key=some_value,some_key_2=some_value_2\n",
        ];
        let expected = Metric::Counter(Counter {
            value: 12345.6,
            sample_rate: None,
        });
        let parsed = expect_metric(&input, expected)?;
        assert_eq!(parsed.msg.name, "some_\tstr");
        Ok(())
    }

    #[test]
    fn test_gauge() -> Result<(), MetricForwarderError> {
        let input = [
            "MONITORING",
            "2017-04-26T10:41:09.023Z",
            "some_str:12345.6|g",
        ];
        let expected = Metric::Gauge(Gauge {
            value: 12345.6,
            sample_rate: None,
        });
        let parsed = expect_metric(&input, expected)?;
        assert_eq!(parsed.msg.name, "some_\tstr");
        Ok(())
    }

    #[test]
    fn test_parse_but_doesnt_have_three_elements_joined_by_tab() -> Result<(), String> {
        let input = ["just two", "things separated by tab"];
        let input_joined = input.join(MONITORING_DELIM);
        let parsed = parse_log(input_joined.as_str());
        match parsed {
            Err(e) => {
                assert_eq!(
                    e,
                    MetricForwarderError::PoorlyFormattedLogLine(input_joined)
                );
                Ok(())
            }
            Ok(_) => Err(String::from("we expected an err here")),
        }
    }

    #[test]
    fn test_couldnt_parse_statsd() -> Result<(), String> {
        let input = [
            "MONITORING",
            "2017-04-26T10:41:09.023Z",
            "some_str:12345.6|fake_metric_type",
        ];
        let input_joined = input.join(MONITORING_DELIM);
        let parsed = parse_log(input_joined.as_str());
        match parsed {
            Err(MetricForwarderError::ParseStringToStatsdError(e, _)) => {
                assert_eq!(e, statsd_parser::ParseError::UnknownMetricType.to_string());
                Ok(())
            }
            _ => Err(String::from("we expected an err here")),
        }
    }
}
