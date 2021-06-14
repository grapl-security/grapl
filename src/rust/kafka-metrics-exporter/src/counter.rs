use metrics::Key;

use crate::metric_message::{
    Counter,
    Label,
};

impl Counter {
    pub fn new(name: impl Into<String>, increment: u64, labels: Vec<Label>) -> Self {
        Self {
            name: name.into(),
            increment,
            labels,
        }
    }
}

impl From<(&Key, u64)> for Counter {
    fn from((key, value): (&Key, u64)) -> Self {
        let labels = key.labels().map(Label::from).collect();
        Counter::new(key.name().to_string(), value, labels)
    }
}
