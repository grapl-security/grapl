pub trait CompletionEventSerializer {
    type CompletedEvent;
    type Output;
    type Error;
    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error>;
}