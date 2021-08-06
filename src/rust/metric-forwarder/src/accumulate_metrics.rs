use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    hash::{
        Hash,
        Hasher,
    },
};

use chrono::{
    DateTime,
    Duration,
    DurationRound,
    FixedOffset,
};
use ordered_float::OrderedFloat;
use rusoto_cloudwatch::{
    Dimension,
    MetricDatum,
};

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
            let mut counts = Vec::with_capacity(count_map.len());
            let mut values = Vec::with_capacity(count_map.len());
            count_map.into_iter().for_each(|(value, count)| {
                values.push(value.into_inner());
                counts.push(count);
            });
            datum.counts = Some(counts);
            datum.values = Some(values);
            datum
        })
        .collect()
}

struct WrappedMetricDatum(MetricDatum);
impl WrappedMetricDatum {
    fn trunc_timestamp(&self) -> DateTime<FixedOffset> {
        let base_ts = DateTime::parse_from_rfc3339(self.0.timestamp.as_ref().unwrap())
            .expect("expected a valid timestamp here");
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
        WrappedDimensions::from(&d.dimensions).hash(state);
        self.trunc_timestamp().hash(state)
    }
}
impl PartialEq for WrappedMetricDatum {
    fn eq(&self, other: &Self) -> bool {
        let self_dims = WrappedDimensions::from(&self.0.dimensions);
        let other_dims = WrappedDimensions::from(&self.0.dimensions);

        return self.0.metric_name == other.0.metric_name
            && self.0.unit == other.0.unit
            && self_dims == other_dims
            && self.trunc_timestamp() == other.trunc_timestamp();
    }
}
impl Eq for WrappedMetricDatum {}

struct WrappedDimensions<'a>(&'a [Dimension]);
impl<'a> WrappedDimensions<'a> {
    /// Turn [(key, value), (key2, value2)] into {key: value, key2: value2}
    /// This is then used for hashing (btreemap chosen because it provides order)
    fn dimensions_as_map(&self) -> BTreeMap<&str, &str> {
        let mut map = BTreeMap::new();
        self.0.iter().for_each(|dim| {
            map.insert(dim.name.as_str(), dim.value.as_str());
        });
        map
    }
}
impl<'a> Hash for WrappedDimensions<'a> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        self.dimensions_as_map().hash(_state);
    }
}

impl<'a> PartialEq for WrappedDimensions<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.dimensions_as_map() == other.dimensions_as_map()
    }
}

static NO_DIMENSIONS: &[Dimension] = &[];
impl<'a> From<&'a Option<Vec<Dimension>>> for WrappedDimensions<'a> {
    fn from(input: &'a Option<Vec<Dimension>>) -> WrappedDimensions<'a> {
        match input {
            Some(dims) => WrappedDimensions(dims),
            None => WrappedDimensions(NO_DIMENSIONS),
        }
    }
}

#[cfg(test)]
mod tests {
    use hmap::hmap;

    use super::*;
    use crate::cloudwatch_send::cw_units;

    const TS: &str = "2020-01-01T01:23:45.000Z";
    const TS_2: &str = "2020-01-01T01:23:49.000Z"; // same min as TS
    const TS_3: &str = "2020-01-01T01:24:00.000Z"; // diff min as TS
    const METRIC_NAME: &str = "metric_name";
    const METRIC_NAME_2: &str = "metric_name_2";

    fn metric(value: f64) -> MetricDatum {
        MetricDatum {
            metric_name: "metric_name".to_string(),
            timestamp: TS.to_string().into(),
            unit: cw_units::COUNT.to_string().into(),
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
            invalid => panic!("Expected exactly 1 metric, but got {:?}", invalid),
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
            invalid => panic!("Expected exactly 2 metrics, but got {:?}", invalid),
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
            invalid => panic!("Expected exactly 2 metrics, but got {:?}", invalid),
        }
    }

    #[test]
    fn test_buckets_same_unit_together() -> Result<(), String> {
        let input = vec![
            metric(1.0),
            {
                let mut m = metric(1.0);
                m.unit = cw_units::MILLIS.to_string().into();
                m
            },
            metric(1.0),
        ];

        let mut output = accumulate_metric_data(input);
        output.sort_by(|a, b| a.unit.cmp(&b.unit));
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
            invalid => panic!("Expected exactly 2 metrics, but got {:?}", invalid),
        }
    }

    #[test]
    #[ignore]
    // This codepath is nondeterministic. It's probably not worth our time to
    // fix, since we are probably moving off of Cloudwatch in the neat future.
    // I'm leaving it here as a "repro of a failure"
    fn test_buckets_diff_tags_differently_nondeterministic_fail() -> Result<(), String> {
        let mut metric_with_diff_dims = metric(1.0);
        metric_with_diff_dims.dimensions = Some(vec![Dimension {
            name: "name2".to_string(),
            value: "value2".to_string(),
        }]);

        let input = vec![
            metric(1.0),
            metric(1.0),
            metric(1.0),
            metric_with_diff_dims.clone(),
            metric_with_diff_dims.clone(),
        ];

        let mut output = accumulate_metric_data(input);
        // the one with a lower count will have name2/value2, and come first.
        output.sort_by_key(|datum| {
            let value: f64 = datum.counts.as_ref().unwrap()[0];
            OrderedFloat(value)
        });

        match &output[..] {
            // So we're expecting this to be [
            //   Metrics with diff dimensions(count=2),
            //   Metrics with default dimensions(count=3) ]
            [datum_diff_dimensions, datum_default_dimensions] => {
                let expected_diff_dims = hmap! {
                    OrderedFloat(1.0f64) => 2f64
                };

                let expected_default_dims = hmap! {
                    OrderedFloat(1.0f64) => 3f64
                };

                assert_eq!(to_count_map(datum_diff_dimensions), expected_diff_dims);
                assert_eq!(
                    to_count_map(datum_default_dimensions),
                    expected_default_dims
                );
                Ok(())
            }
            invalid => panic!("Expected exactly 2 metrics, but got {:?}", invalid),
        }
    }
}
