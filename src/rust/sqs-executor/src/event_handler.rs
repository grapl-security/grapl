use std::fmt::Debug;

use async_trait::async_trait;

use crate::{cache::Cacheable, errors::CheckedError, event_status::EventStatus};

#[derive(Default)]
pub struct CompletedEvents {
    pub identities: Vec<(Vec<u8>, EventStatus)>,
}

impl CompletedEvents {
    pub fn clear(&mut self) {
        self.identities.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.identities.is_empty()
    }

    pub fn len(&self) -> usize {
        self.identities.len()
    }

    pub fn add_identity(&mut self, identity: impl Cacheable, r: impl Into<EventStatus>) {
        self.identities.push((identity.identity(), r.into()));
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
