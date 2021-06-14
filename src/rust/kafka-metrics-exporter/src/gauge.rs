use metrics::{
    GaugeValue,
    Key,
};

use crate::metric_message::{
    gauge::GaugeType,
    Gauge,
    Label,
};

fn split_gauge_value(gauge_value: &GaugeValue) -> (i32, f64) {
    match gauge_value {
        GaugeValue::Absolute(v) => (GaugeType::Absolute as i32, *v),
        GaugeValue::Increment(v) => (GaugeType::Increment as i32, *v),
        GaugeValue::Decrement(v) => (GaugeType::Decrement as i32, *v),
    }
}

impl Gauge {
    pub fn new(name: impl Into<String>, value: GaugeValue, labels: Vec<Label>) -> Self {
        let (gauge_type, value) = split_gauge_value(&value);
        Self {
            name: name.into(),
            gauge_type,
            value,
            labels,
        }
    }
}

impl From<(&Key, GaugeValue)> for Gauge {
    fn from((key, value): (&Key, GaugeValue)) -> Self {
        let labels = key.labels().map(Label::from).collect();
        Gauge::new(key.name().to_string(), value, labels)
    }
}
