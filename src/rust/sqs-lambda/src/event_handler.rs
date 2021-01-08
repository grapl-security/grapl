use async_trait::async_trait;
use std::fmt::Debug;

use crate::cache::Cacheable;

pub enum Completion<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    Total(T),
    Partial((T, E)),
    Error(E),
}

pub struct OutputEvent<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    pub completed_event: Completion<T, E>,
    pub identities: Vec<Vec<u8>>,
}

impl<T, E> OutputEvent<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    pub fn new(completed_event: Completion<T, E>) -> Self {
        Self {
            completed_event,
            identities: Vec::new(),
        }
    }

    pub fn add_identity(&mut self, identity: impl Cacheable) {
        self.identities.push(identity.identity())
    }
}

#[async_trait]
pub trait EventHandler {
    type InputEvent;
    type OutputEvent: Clone + Send + Sync + 'static;
    type Error: Debug + Send + Sync + 'static;

    async fn handle_event(
        &mut self,
        input: Self::InputEvent,
    ) -> OutputEvent<Self::OutputEvent, Self::Error>;
}
