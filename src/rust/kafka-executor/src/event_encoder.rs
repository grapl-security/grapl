pub trait EventSerializer {
    type OutputEvent;
    type Error: std::error::Error;
    fn serialize(&mut self, event: Self::OutputEvent) -> Result<Vec<u8>, Self::Error>;
}
