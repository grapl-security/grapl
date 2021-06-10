use metrics::Key;

use crate::metric_message::{
    Histogram,
    Label,
};

impl Histogram {
    pub fn new(name: impl Into<String>, value: f64, labels: Vec<Label>) -> Self {
        Self {
            name: name.into(),
            value,
            labels,
        }
    }
}

impl From<(&Key, f64)> for Histogram {
    fn from((key, value): (&Key, f64)) -> Self {
        let labels = key.labels().map(Label::from).collect();
        Histogram::new(key.name().to_string(), value, labels)
    }
}
