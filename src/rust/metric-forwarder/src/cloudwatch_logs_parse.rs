use aws_lambda_events::event::cloudwatch_logs::{CloudwatchLogsData, CloudwatchLogsLogEvent};
use rayon::prelude::*;
use statsd_parser;

use crate::error::MetricForwarderError;

#[derive(Debug)]
pub struct Stat {
    pub msg: statsd_parser::Message,
    pub timestamp: String,
}

const MONITORING_PREFIX: &'static str = "MONITORING|";
const CLOUDWATCH_LOGS_DELIM: &'static str = "\t";

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
    a typical CloudWatch Logs log message looks like this:
        "2017-04-26T10:41:09.023Z\tdb95c6da-2a6c-11e7-9550-c91b65931beb\tmy log message\n"
           <ts>                  \t         <dont care>                \t <log contents>\n
    */
    let split: Vec<&str> = log_str.splitn(3, CLOUDWATCH_LOGS_DELIM).collect();
    match &split[..] {
        [timestamp, _, statsd_component] => {
            let stat = parse_statsd_component(statsd_component).map(|statsd_message| Stat {
                timestamp: timestamp.to_string(),
                msg: statsd_message,
            });
            stat
        }
        _ => Err(MetricForwarderError::PoorlyFormattedLogLine(
            log_str.to_string(),
        )),
    }
}

fn parse_statsd_component(log_str: &str) -> Result<statsd_parser::Message, MetricForwarderError> {
    let stripped = log_str.strip_prefix(MONITORING_PREFIX);
    match stripped {
        Some(s) => statsd_parser::parse(s).map_err(|parse_err| {
            MetricForwarderError::ParseStringToStatsdError(parse_err.to_string())
        }),
        _ => Err(MetricForwarderError::PoorlyFormattedLogLine(
            log_str.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::cloudwatch_logs_parse::parse_log;
    use crate::cloudwatch_logs_parse::CLOUDWATCH_LOGS_DELIM;
    use crate::error::MetricForwarderError;
    use statsd_parser::Counter;
    use statsd_parser::Metric;

    #[test]
    fn test_parse_one_log() -> Result<(), MetricForwarderError> {
        let input = [
            "2017-04-26T10:41:09.023Z",
            "db95c6da-2a6c-11e7-9550-c91b65931beb",
            // you'll note I threw a gross, extra \t in the metric name as an edge case
            "MONITORING|some_\tstr:12345.6|c|#some_key=some_value,some_key_2=some_value_2\n",
        ];
        let input_joined = input.join(CLOUDWATCH_LOGS_DELIM);
        let parsed = parse_log(input_joined.as_str())?;
        assert_eq!(parsed.msg.name, "some_\tstr");
        assert_eq!(
            parsed.msg.metric,
            Metric::Counter(Counter {
                value: 12345.6,
                sample_rate: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_parse_but_doesnt_have_three_elements_joined_by_tab() -> Result<(), String> {
        let input = ["just two", "things separated by tab"];
        let input_joined = input.join(CLOUDWATCH_LOGS_DELIM);
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
    fn test_no_monitoring_prefix() -> Result<(), String> {
        let input = [
            "2017-04-26T10:41:09.023Z",
            "db95c6da-2a6c-11e7-9550-c91b65931beb",
            "some_str:12345.6|c|#some_key=some_value,some_key_2=some_value_2\n",
        ];
        let input_joined = input.join(CLOUDWATCH_LOGS_DELIM);
        let parsed = parse_log(input_joined.as_str());
        match parsed {
            Err(e) => {
                assert_eq!(
                    e,
                    MetricForwarderError::PoorlyFormattedLogLine(input[2].to_string())
                );
                Ok(())
            }
            Ok(_) => Err(String::from("we expected an err here")),
        }
    }

    #[test]
    fn test_couldnt_parse_statsd() -> Result<(), String> {
        let input = [
            "2017-04-26T10:41:09.023Z",
            "db95c6da-2a6c-11e7-9550-c91b65931beb",
            "MONITORING|some_str:12345.6|fake_metric_type",
        ];
        let input_joined = input.join(CLOUDWATCH_LOGS_DELIM);
        let parsed = parse_log(input_joined.as_str());
        match parsed {
            Err(MetricForwarderError::ParseStringToStatsdError(e)) => {
                assert_eq!(e, statsd_parser::ParseError::UnknownMetricType.to_string());
                Ok(())
            }
            _ => Err(String::from("we expected an err here")),
        }
    }
}
