impl From<&metrics::Label> for crate::metric_message::Label {
    fn from(label: &metrics::Label) -> Self {
        Self {
            key: label.key().to_owned(),
            value: label.value().to_owned(),
        }
    }
}
