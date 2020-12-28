use std::fmt::Debug;
use async_trait::async_trait;
use crate::errors::CheckedError;
use crate::cache::Cacheable;

#[derive(Default)]
pub struct CompletedEvents {
    identities: Vec<Vec<u8>>,
}

impl CompletedEvents {
    pub fn clear(&mut self) {
        self.identities.clear();
    }

    pub fn add_identity(&mut self, identity: impl Cacheable) {
        self.identities.push(identity.identity());
    }
}


#[async_trait]
pub trait EventHandler {
    type InputEvent;
    type OutputEvent: Clone + Send + Sync + 'static;
    type Error: Debug + CheckedError + Send + Sync + 'static;

    async fn handle_event(
        &mut self,
        input: Self::InputEvent,
        identities: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>>;
}
