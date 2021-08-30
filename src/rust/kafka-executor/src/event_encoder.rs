pub trait EventSerializer {
    type OutputEvent;
    type Error: std::error::Error;
    fn encode_event(&self, event: Self::OutputEvent) -> Result<Vec<u8>, Self::Error>;
}
