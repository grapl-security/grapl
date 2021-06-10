use crate::metric_message::Counter;
use crate::metric_message::Label;
use metrics::Key;

impl Counter {
    pub fn new(
        name: impl Into<String>,
        value: u64,
        labels: Vec<Label>,
    ) -> Self {
        Self {
            metric_type: "counter".to_string(),
            name: name.into(),
            value,
            labels,
        }
    }
}

impl From<(&Key, u64)> for Counter {
    fn from((key, value): (&Key, u64)) -> Self {
        let labels = key.labels().map(Label::from).collect();
        Counter::new(
            key.name().to_string(),
            value,
            labels,
        )
    }
}
