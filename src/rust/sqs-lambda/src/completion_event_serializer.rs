pub trait CompletionEventSerializer {
    type CompletedEvent;
    type Output;
    type Error;
    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error>;
}


impl<F, CE, O, Err> CompletionEventSerializer for F
    where F: Fn(&[Self::CompetedEvent]) -> Result<Vec<Self::Output>, Self::Error>
{
    type CompletedEvent = CE;
    type Output = O;
    type Error = Err;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        (self)(completed_events)
    }
}