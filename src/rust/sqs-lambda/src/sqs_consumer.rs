use std::io::Stdout;
use std::marker::PhantomData;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use lambda_runtime::Context;
use log::{debug, error};
use rusoto_sqs::{ReceiveMessageRequest, Sqs};
use rusoto_sqs::Message as SqsMessage;
use tokio::sync::mpsc::{channel, Sender};
use tracing::instrument;

use grapl_observe::metric_reporter::MetricReporter;

use crate::completion_handler::CompletionHandler;
use crate::consumer::Consumer;
use crate::event_processor::EventProcessorActor;
use grapl_observe::timers::time_fut_ms;

#[derive(Debug, Clone, Default)]
pub struct ConsumePolicyBuilder {
    deadline: Option<i64>,
    stop_at: Option<Duration>,
    max_empty_receives: Option<u16>,
}

impl ConsumePolicyBuilder {
    pub fn with_max_empty_receives(mut self, arg: u16) -> Self {
        self.max_empty_receives = Some(arg);
        self
    }

    pub fn with_stop_at(mut self, arg: Duration) -> Self {
        self.stop_at = Some(arg);
        self
    }

    pub fn build(self, deadline: impl IntoDeadline) -> ConsumePolicy {
        ConsumePolicy::new(
            deadline,
            self.stop_at.unwrap_or_else(|| Duration::from_secs(10)),
            self.max_empty_receives.unwrap_or_else(|| 1),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ConsumePolicy {
    deadline: i64,
    stop_at: Duration,
    max_empty_receives: u16,
    empty_receives: u16,
}

pub trait IntoDeadline {
    fn into_deadline(self) -> i64;
}

impl IntoDeadline for Context {
    fn into_deadline(self) -> i64 {
        self.deadline
    }
}

impl IntoDeadline for i64 {
    fn into_deadline(self) -> i64 {
        self
    }
}

impl ConsumePolicy {
    pub fn new(deadline: impl IntoDeadline, stop_at: Duration, max_empty_receives: u16) -> Self {
        Self {
            deadline: deadline.into_deadline(),
            stop_at,
            max_empty_receives,
            empty_receives: 0,
        }
    }

    pub fn get_time_remaining_millis(&self) -> i64 {
        self.deadline - Utc::now().timestamp_millis()
    }

    pub fn should_consume(&self) -> bool {
        (self.stop_at.as_millis() <= self.get_time_remaining_millis() as u128)
            && self.empty_receives <= self.max_empty_receives
    }

    pub fn register_received(&mut self, any: bool) {
        if any {
            self.empty_receives = 0;
        } else {
            self.empty_receives += 1;
        }
    }
}

pub struct SqsConsumer<S, CH>
    where
        S: Sqs + Send + Sync + 'static,
        CH: CompletionHandler + Clone + Send + Sync + 'static,
{
    sqs_client: S,
    queue_url: String,
    stored_events: Vec<SqsMessage>,
    consume_policy: ConsumePolicy,
    completion_handler: CH,
    metric_reporter: MetricReporter<Stdout>,
    shutdown_subscriber: Option<tokio::sync::oneshot::Sender<()>>,
    self_actor: Option<SqsConsumerActor<S, CH>>,
}

impl<S, CH> SqsConsumer<S, CH>
    where
        S: Sqs + Send + Sync + 'static,
        CH: CompletionHandler + Clone + Send + Sync + 'static,
{
    pub fn new(
        sqs_client: S,
        queue_url: String,
        consume_policy: ConsumePolicy,
        completion_handler: CH,
        metric_reporter: MetricReporter<Stdout>,
        shutdown_subscriber: tokio::sync::oneshot::Sender<()>,
    ) -> SqsConsumer<S, CH>
        where
            S: Sqs,
    {
        Self {
            sqs_client,
            queue_url,
            stored_events: Vec::with_capacity(20),
            consume_policy,
            completion_handler,
            shutdown_subscriber: Some(shutdown_subscriber),
            metric_reporter,
            self_actor: None,
        }
    }
}

impl<S: Sqs + Send + Sync + 'static, CH: CompletionHandler + Clone + Send + Sync + 'static>
SqsConsumer<S, CH>
{
    #[instrument(skip(self))]
    pub async fn batch_get_events(&mut self, wait_time_seconds: i64) -> eyre::Result<Vec<SqsMessage>> {
        debug!("Calling receive_message");
        let visibility_timeout = Duration::from_millis(self.consume_policy.get_time_remaining_millis() as u64).as_secs() + 1;
        let recv = self.sqs_client.receive_message(ReceiveMessageRequest {
            max_number_of_messages: Some(10),
            queue_url: self.queue_url.clone(),
            wait_time_seconds: Some(wait_time_seconds),
            visibility_timeout: Some(visibility_timeout as i64),
            ..Default::default()
        });

        let recv = tokio::time::timeout(Duration::from_secs(wait_time_seconds as u64 + 2), recv);
        let (recv, ms) = time_fut_ms(recv).await;
        let recv = recv??;
        debug!("Called receive_message : {:?}", recv);
        self.metric_reporter.histogram("sqs_consumer.receive_message.ms", ms as f64, &[])
            .unwrap_or_else(|e| error!("failed to report sqs_consumer.receive_message.ms: {:?}", e));

        self.metric_reporter.counter(
            "sqs_consumer.receive_message.count",
            recv.messages.as_ref().map(|m| m.len()).unwrap_or_default() as f64,
            None
        )
            .unwrap_or_else(|e| error!("failed to report sqs_consumer.receive_message.count: {:?}", e));

        Ok(recv.messages.unwrap_or(vec![]))
    }
}

#[derive_aktor::derive_actor]
impl<S: Sqs + Send + Sync + 'static, CH: CompletionHandler + Clone + Send + Sync + 'static>
SqsConsumer<S, CH>
{
    #[instrument(skip(self, event_processor))]
    pub async fn get_new_event(&mut self, event_processor: EventProcessorActor<SqsMessage>) {
        debug!("New event request");
        let should_consume = self.consume_policy.should_consume();

        if self.stored_events.is_empty() && should_consume {
            let new_events = match self.batch_get_events(1).await {
                Ok(new_events) => new_events,
                Err(e) => {
                    error!("Failed to get new events with: {:?}", e);
                    tokio::time::delay_for(Duration::from_secs(1)).await;
                    self.self_actor
                        .clone()
                        .unwrap()
                        .get_next_event(event_processor)
                        .await;

                    // Register the empty receive on error
                    self.consume_policy.register_received(false);
                    return;
                }
            };

            self.consume_policy
                .register_received(!new_events.is_empty());
            self.stored_events.extend(new_events);
        }

        if !should_consume {
            debug!("Done consuming, forcing ack");
            let (tx, shutdown_notify) = tokio::sync::oneshot::channel();

            // If we're past the point of consuming it's time to start acking
            self.completion_handler.ack_all(Some(tx)).await;

            let _ = shutdown_notify.await;
            debug!("Ack complete");
        }

        if self.stored_events.is_empty() && !should_consume {
            debug!("No more events to process, and we should not consume more");
            let shutdown_subscriber = std::mem::replace(&mut self.shutdown_subscriber, None);
            match shutdown_subscriber {
                Some(shutdown_subscriber) => {
                    shutdown_subscriber.send(()).unwrap();
                }
                None => debug!("Attempted to shut down with empty shutdown_subscriber"),
            };

            event_processor.stop_processing().await;
            drop(event_processor);
            return;
        }

        if let Some(next_event) = self.stored_events.pop() {
            debug!("Sending next event to processor");
            event_processor.process_event(next_event).await;
            debug!("Sent next event to processor");
        } else {
            tokio::time::delay_for(Duration::from_millis(50)).await;
            debug!("No events to send to processor");
            self.self_actor
                .clone()
                .unwrap()
                .get_next_event(event_processor)
                .await;
        }
    }

    pub async fn _p(&self, __p: PhantomData<(S, CH)>) {}
}

#[async_trait]
impl<S, CH> Consumer<SqsMessage> for SqsConsumerActor<S, CH>
    where
        S: Sqs + Send + Sync + 'static,
        CH: CompletionHandler + Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, event_processor))]
    async fn get_next_event(&self, event_processor: EventProcessorActor<SqsMessage>) {
        let msg = SqsConsumerMessage::get_new_event { event_processor };
        self.queue_len
            .clone()
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut sender = self.sender.clone();
        tokio::task::spawn(async move {
            if let Err(e) = sender.send(msg).await {
                panic!(
                    concat!(
                    "Receiver has failed with {}, propagating error. ",
                    "SqsConsumerActor.get_next_event"
                    ),
                    e
                )
            }
        });
    }
}
