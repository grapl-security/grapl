use chrono::{DateTime, Duration, DurationRound, FixedOffset};

use ordered_float::OrderedFloat;

use rusoto_cloudwatch::{Dimension, MetricDatum};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

type CountMap = HashMap<OrderedFloat<f64>, f64>;
type DatumToCountMap = HashMap<WrappedMetricDatum, CountMap>;

pub fn accumulate_metric_data(input: Vec<MetricDatum>) -> Vec<MetricDatum> {
    let mut datum_to_count: DatumToCountMap = HashMap::new();
    /*
     * This section of the code will convert input of (boiled down for simplicity)
     *   [MetricDatum('metric', 1.0), MetricDatum('metric', 5.0), MetricDatum('metric', 1.0)]
     * into
     * {
     *   MetricDatum('metric', _): {
     *     // value -> num_occurences
     *     1.0: 2,
     *     5.0: 1
     *   }
     * }
     */
    input.into_iter().for_each(|datum| {
        let value = &datum.value.unwrap();
        let count_map = datum_to_count
            .entry(WrappedMetricDatum(datum))
            .or_insert(HashMap::new());
        let count = count_map.entry(OrderedFloat(*value)).or_insert(0f64);
        (*count) += 1f64;
    });

    /*
     * This section will take the above `datum_to_count_map` and convert it into a flat vec<MetricDatum>, where each
     * MetricDatum internally contains the {1.0: 2, 5.0: 1} in its .counts and .values
     */
    datum_to_count
        .into_iter()
        .map(|(WrappedMetricDatum(mut datum), count_map)| {
            // Erase the scalar "value", then fill "counts" and "values" vecs
            datum.value = None;
            let mut counts = vec![];
            let mut values: Vec<f64> = vec![];
            count_map.into_iter().for_each(|(value, count)| {
                values.push(value.into_inner());
                counts.push(count);
            });
            datum.counts = counts.into();
            datum.values = values.into();
            datum
        })
        .collect()
}

struct WrappedMetricDatum(MetricDatum);
impl WrappedMetricDatum {
    fn trunc_timestamp(&self) -> DateTime<FixedOffset> {
        let base_ts = DateTime::parse_from_rfc3339(self.0.timestamp.as_ref().unwrap()).unwrap();
        let truncated = base_ts.duration_trunc(Duration::minutes(1)).unwrap();
        truncated
    }
}
impl Hash for WrappedMetricDatum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // group-by-<MetricName + Rounded Timestamp + Unit + Dimensions>
        let d = &self.0;
        d.metric_name.hash(state);
        d.unit.hash(state);
        d.dimensions.as_ref().map(|dims| {
            dims.iter().for_each(|dim| {
                dim.name.hash(state);
                dim.value.hash(state);
            })
        });
        self.trunc_timestamp().hash(state)
    }
}
impl PartialEq for WrappedMetricDatum {
    fn eq(&self, other: &Self) -> bool {
        return self.0.metric_name == other.0.metric_name
            && self.0.unit == other.0.unit
            && self.0.dimensions == other.0.dimensions
            && self.trunc_timestamp() == other.trunc_timestamp();
    }
}
impl Eq for WrappedMetricDatum {}

struct WrappedDimensions<'a>(&'a Vec<Dimension>);
impl<'a> Hash for WrappedDimensions<'a> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        let _dims = self.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloudwatch_send::units;
    use hmap::hmap;

    const TS: &str = "2020-01-01T01:23:45.000Z";
    const TS_2: &str = "2020-01-01T01:23:49.000Z"; // same min as TS
    const TS_3: &str = "2020-01-01T01:24:00.000Z"; // diff min as TS
    const METRIC_NAME: &str = "metric_name";
    const METRIC_NAME_2: &str = "metric_name_2";

    fn metric(value: f64) -> MetricDatum {
        MetricDatum {
            metric_name: "metric_name".to_string(),
            timestamp: TS.to_string().into(),
            unit: units::COUNT.to_string().into(),
            value: value.into(),
            counts: None,
            values: None,
            dimensions: vec![Dimension {
                name: "name".to_string(),
                value: "value".to_string(),
            }]
            .into(),
            statistic_values: None,
            storage_resolution: None,
        }
    }

    fn to_count_map(datum: &MetricDatum) -> CountMap {
        use itertools::zip;
        let mut results: CountMap = HashMap::new();
        for (count, value) in zip(
            datum.counts.as_ref().unwrap(),
            datum.values.as_ref().unwrap(),
        ) {
            results.insert(OrderedFloat(*value), *count);
        }
        results
    }

    #[test]
    fn test_all_same_except_value() -> Result<(), String> {
        let input = vec![metric(1.0), metric(5.0), metric(1.0)];

        let output = accumulate_metric_data(input);
        match &output[..] {
            [only_one_datum] => {
                let expected = hmap! {
                    OrderedFloat(1f64) => 2f64,
                    OrderedFloat(5f64) => 1f64
                };
                assert_eq!(to_count_map(only_one_datum), expected);
                Ok(())
            }
            _ => Err(">1 metric".to_string()),
        }
    }

    #[test]
    fn test_buckets_diff_names_differently() -> Result<(), String> {
        let input = vec![metric(1.0), {
            let mut m = metric(1.0);
            m.metric_name = METRIC_NAME_2.to_string();
            m
        }];

        let mut output = accumulate_metric_data(input);
        output.sort_by(|a, b| a.metric_name.cmp(&b.metric_name));
        match &output[..] {
            [datum_0, datum_1] => {
                let expected_for_both = hmap! {
                    OrderedFloat(1f64) => 1f64
                };
                assert_eq!(to_count_map(datum_0), expected_for_both);
                assert_eq!(to_count_map(datum_1), expected_for_both);
                assert_eq!(datum_0.metric_name, METRIC_NAME);
                assert_eq!(datum_1.metric_name, METRIC_NAME_2);
                Ok(())
            }
            _ => Err("must be exactly 2 metrics".to_string()),
        }
    }

    #[test]
    fn test_buckets_same_minute_together() -> Result<(), String> {
        let input = vec![
            metric(1.0),
            {
                let mut m = metric(1.0);
                m.timestamp = TS_2.to_string().into();
                m
            },
            {
                let mut m = metric(1.0);
                m.timestamp = TS_3.to_string().into();
                m
            },
        ];

        let mut output = accumulate_metric_data(input);
        output.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        match &output[..] {
            [datum_0, datum_1] => {
                let expected_for_first_min = hmap! {
                    OrderedFloat(1f64) => 2f64
                };

                let expected_for_second_min = hmap! {
                    OrderedFloat(1f64) => 1f64
                };
                assert_eq!(to_count_map(datum_0), expected_for_first_min);
                assert_eq!(to_count_map(datum_1), expected_for_second_min);
                assert_eq!(datum_0.timestamp, TS.to_string().into());
                assert_eq!(datum_1.timestamp, TS_3.to_string().into());
                Ok(())
            }
            _ => Err("must be exactly 2 metrics".to_string()),
        }
    }

    #[test]
    fn test_buckets_same_unit_together() -> Result<(), String> {
        let input = vec![
            metric(1.0),
            {
                let mut m = metric(1.0);
                m.unit = units::MILLIS.to_string().into();
                m
            },
            metric(1.0),
        ];

        let mut output = accumulate_metric_data(input);
        output.sort_by(|a, b| a.unit.cmp(&b.timestamp));
        match &output[..] {
            [datum_count, datum_millis] => {
                let expected_for_count = hmap! {
                    OrderedFloat(1f64) => 2f64
                };

                let expected_for_millis = hmap! {
                    OrderedFloat(1f64) => 1f64
                };
                assert_eq!(to_count_map(datum_count), expected_for_count);
                assert_eq!(to_count_map(datum_millis), expected_for_millis);
                Ok(())
            }
            _ => Err("must be exactly 2 metrics".to_string()),
        }
    }
}
