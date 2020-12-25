use async_trait::async_trait;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use thiserror::Error;

use crate::cache::NopCache;
use crate::completion_event_serializer::CompletionEventSerializer;
use crate::event_retriever::S3PayloadRetriever;
use crate::s3_event_emitter::S3EventEmitter;

pub struct MyEvent {}
#[derive(Debug, Clone)]
pub struct MyOutput {}
pub struct MyGenerator {}


#[derive(Clone, Debug, Default)]
pub struct MySerializer {}


#[derive(Error, Debug)]
pub enum MySerializerError {
    // #[error("Failed to receive sqs messages")]
    // SqsReceiveError(#[from] rusoto_core::RusotoError<ReceiveMessageError>),
}


impl CompletionEventSerializer for MySerializer {
    type CompletedEvent = MyOutput;
    type Output = Vec<u8>;
    type Error = MySerializerError;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        Ok(vec![])
    }
}

#[derive(Error, Debug)]
pub enum MyHandlerError {
    #[error("Bug")]
    Bug,
}


impl crate::errors::CheckedError for MyHandlerError {
    fn error_type(&self) -> crate::errors::Recoverable {
        crate::errors::Recoverable::Transient
    }
}


#[async_trait]
impl crate::event_handler::EventHandler for MyGenerator {
    type InputEvent = Vec<MyEvent>;
    type OutputEvent = MyOutput;
    type Error = MyHandlerError;

    async fn handle_event(&mut self, input: Self::InputEvent) -> Result<Self::OutputEvent, Result<Self::OutputEvent, Self::Error>> {
        unimplemented!()
    }
}


pub async fn fake_main() {
    let sqs_client: SqsClient = unimplemented!();
    let s3_client: S3Client = unimplemented!();
    let mut fake_generator = &mut [MyGenerator {}; 10];

    let mut serializer = &mut [MySerializer {}; 10];

    let s3_emitter = &mut [S3EventEmitter::new(
        s3_client,
        "output_bucket",
        |_| "s".to_owned(),
        move |_, _| async move {Ok(())}
    ); 10];

    let s3_payload_retriever= &mut [S3PayloadRetriever::new(
        |_| S3Client::new(unimplemented!()),
        |_| Ok(vec![MyEvent {}]),
        unimplemented!(),
    ); 10];

    let cache = &mut [NopCache{}; 10];

    crate::process_loop(
        "queue_url".to_owned(),
        cache,
        sqs_client.clone(),
        fake_generator,
        s3_payload_retriever,
        s3_emitter,
        serializer,
    ).await;
}