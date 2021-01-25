use std::fmt::Debug;
use std::time::{Duration, Instant};

use log::*;
use rusoto_sqs::Message as SqsMessage;
use rusoto_sqs::{DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, Sqs};
use tokio::sync::mpsc::{channel, Sender};

use crate::cache::Cache;
use crate::completion_event_serializer::CompletionEventSerializer;
use crate::event_emitter::EventEmitter;
use crate::event_handler::{Completion, OutputEvent};
use aktors::actor::Actor;
use async_trait::async_trait;

use crate::completion_handler::CompletionHandler;
use color_eyre::Help;
use grapl_observe::metric_reporter::MetricReporter;
use grapl_observe::timers::time_fut_ms;
use std::io::Stdout;

#[derive(Debug, Clone)]
pub struct CompletionPolicy {
    max_messages: u16,
    max_time_between_flushes: Duration,
    last_flush: Instant,
}

impl CompletionPolicy {
    pub fn new(max_messages: u16, max_time_between_flushes: Duration) -> Self {
        Self {
            max_messages,
            max_time_between_flushes,
            last_flush: Instant::now(),
        }
    }

    pub fn should_flush(&self, cur_messages: u16) -> bool {
        cur_messages >= self.max_messages
            || Instant::now()
                .checked_duration_since(self.last_flush)
                .unwrap()
                >= self.max_time_between_flushes
    }

    pub fn set_last_flush(&mut self) {
        self.last_flush = Instant::now();
    }
}

pub struct SqsCompletionHandler<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
    CPE: Debug + Send + Sync + 'static,
    CP: CompletionEventSerializer<CompletedEvent = CE, Output = Payload, Error = CPE>
        + Send
        + Sync
        + 'static,
    Payload: Send + Sync + 'static,
    CE: Send + Sync + Clone + 'static,
    EE: EventEmitter<Event = Payload> + Send + Sync + 'static,
    OA: Fn(SqsCompletionHandlerActor<CE, ProcErr, SqsT>, Result<String, String>)
        + Send
        + Sync
        + 'static,
    CacheT: Cache + Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
{
    sqs_client: SqsT,
    queue_url: String,
    completed_events: Vec<CE>,
    identities: Vec<Vec<u8>>,
    completed_messages: Vec<SqsMessage>,
    completion_serializer: CP,
    event_emitter: EE,
    completion_policy: CompletionPolicy,
    on_ack: OA,
    self_actor: Option<SqsCompletionHandlerActor<CE, ProcErr, SqsT>>,
    cache: CacheT,
    metric_reporter: MetricReporter<Stdout>,
    _p: std::marker::PhantomData<ProcErr>,
}

impl<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>
    SqsCompletionHandler<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
    CPE: Debug + Send + Sync + 'static,
    CP: CompletionEventSerializer<CompletedEvent = CE, Output = Payload, Error = CPE>
        + Send
        + Sync
        + 'static,
    Payload: Send + Sync + 'static,
    CE: Send + Sync + Clone + 'static,
    EE: EventEmitter<Event = Payload> + Send + Sync + 'static,
    OA: Fn(SqsCompletionHandlerActor<CE, ProcErr, SqsT>, Result<String, String>)
        + Send
        + Sync
        + 'static,
    CacheT: Cache + Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
{
    pub fn new(
        sqs_client: SqsT,
        queue_url: String,
        completion_serializer: CP,
        event_emitter: EE,
        completion_policy: CompletionPolicy,
        on_ack: OA,
        cache: CacheT,
        metric_reporter: MetricReporter<Stdout>,
    ) -> Self {
        Self {
            sqs_client,
            queue_url,
            completed_events: Vec::with_capacity(completion_policy.max_messages as usize),
            identities: Vec::with_capacity(completion_policy.max_messages as usize),
            completed_messages: Vec::with_capacity(completion_policy.max_messages as usize),
            completion_serializer,
            event_emitter,
            completion_policy,
            on_ack,
            self_actor: None,
            cache,
            metric_reporter,
            _p: std::marker::PhantomData,
        }
    }
}

async fn retry<F, T, E>(max_tries: u32, f: impl Fn() -> F) -> color_eyre::Result<T>
where
    T: Send,
    F: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut backoff = 2;
    let mut errs: Result<T, _> = Err(eyre::eyre!("wait_loop failed"));
    for i in 0..max_tries {
        match (f)().await {
            Ok(t) => return Ok(t),
            Err(e) => {
                errs = errs.error(e);
            }
        };

        tokio::time::delay_for(Duration::from_millis(backoff)).await;
        backoff *= i as u64;
    }

    errs
}

impl<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>
    SqsCompletionHandler<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
    CPE: Debug + Send + Sync + 'static,
    CP: CompletionEventSerializer<CompletedEvent = CE, Output = Payload, Error = CPE>
        + Send
        + Sync
        + 'static,
    Payload: Send + Sync + 'static,
    CE: Send + Sync + Clone + 'static,
    EE: EventEmitter<Event = Payload> + Send + Sync + 'static,
    OA: Fn(SqsCompletionHandlerActor<CE, ProcErr, SqsT>, Result<String, String>)
        + Send
        + Sync
        + 'static,
    CacheT: Cache + Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self))]
    pub async fn ack_message(&mut self, sqs_message: SqsMessage) {
        self.completed_messages.push(sqs_message);
        if self
            .completion_policy
            .should_flush(self.completed_events.len() as u16)
        {
            self.ack_all(None).await;
            self.completion_policy.set_last_flush();
        }
    }

    #[tracing::instrument(skip(self, completed))]
    pub async fn mark_complete(
        &mut self,
        sqs_message: SqsMessage,
        completed: OutputEvent<CE, ProcErr>,
    ) {
        match completed.completed_event {
            Completion::Total(ce) => {
                info!("Marking all events complete - total success");
                self.completed_events.push(ce);
                self.completed_messages.push(sqs_message);
                self.identities.extend(completed.identities);
            }
            Completion::Partial((ce, err)) => {
                warn!("EventHandler was only partially successful: {:?}", err);
                self.completed_events.push(ce);
                self.identities.extend(completed.identities);
            }
            Completion::Error(e) => {
                warn!("Event handler failed: {:?}", e);
            }
        };

        info!(
            "Marked event complete. {} completed events, {} completed messages",
            self.completed_events.len(),
            self.completed_messages.len(),
        );

        if self
            .completion_policy
            .should_flush(self.completed_events.len() as u16)
        {
            self.ack_all(None).await;
            self.completion_policy.set_last_flush();
        }
    }

    #[tracing::instrument(skip(self, notify))]
    pub async fn ack_all(&mut self, notify: Option<tokio::sync::oneshot::Sender<()>>) {
        debug!("Flushing completed events");

        let serialized_event = self
            .completion_serializer
            .serialize_completed_events(&self.completed_events[..]);

        let serialized_event = match serialized_event {
            Ok(serialized_event) => serialized_event,
            Err(e) => {
                // We should emit a failure, but ultimately we just have to not ack these messages
                self.completed_events.clear();
                self.completed_messages.clear();

                panic!("Serializing events failed: {:?}", e);
            }
        };

        // todo: Retrying here would be a good idea, panicking is basically shutting this entire system down
        debug!("Emitting events");
        let (_emit_res, _ms) = time_fut_ms(self.event_emitter.emit_event(serialized_event)).await;
        // emit_res.expect("Failed to emit event");
        //
        // self.metric_reporter.histogram("sqs_completion_handler.emit_event.ms", ms as f64, &[])
        //     .unwrap_or_else(|e| error!("failed to report sqs_completion_handler.emit_event.ms: {:?}", e));

        for identity in self.identities.drain(..) {
            if let Err(e) = self.cache.store(identity).await {
                warn!("Failed to cache with: {:?}", e);
            }
        }

        self.metric_reporter
            .histogram(
                "sqs_completion_handler.completed_messages.count",
                self.completed_messages.len() as f64,
                &[],
            )
            .unwrap_or_else(|e| {
                error!(
                    "failed to report sqs_completion_handler.completed_messages.count: {:?}",
                    e
                )
            });

        let mut acks = vec![];
        for chunk in self.completed_messages.chunks(10) {
            let msg_ids: Vec<String> = chunk
                .iter()
                .map(|msg| msg.message_id.clone().unwrap())
                .collect();

            let entries: Vec<_> = chunk
                .iter()
                .map(|msg| DeleteMessageBatchRequestEntry {
                    id: msg.message_id.clone().unwrap(),
                    receipt_handle: msg.receipt_handle.clone().expect("Message missing receipt"),
                })
                .collect();

            let res = retry(10, || async {
                let dmb = self
                    .sqs_client
                    .delete_message_batch(DeleteMessageBatchRequest {
                        entries: entries.clone(),
                        queue_url: self.queue_url.clone(),
                    });

                tokio::time::timeout(Duration::from_millis(250), dmb).await
            });

            let (res, ms) = time_fut_ms(res).await;
            self.metric_reporter
                .histogram(
                    "sqs_completion_handler.delete_message_batch.ms",
                    ms as f64,
                    &[],
                )
                .unwrap_or_else(|e| {
                    error!(
                        "failed to report sqs_completion_handler.delete_message_batch.ms: {:?}",
                        e
                    )
                });

            match res {
                Ok(dmb) => acks.push((dmb, msg_ids)),
                Err(e) => warn!("Failed to delete message, timed out: {:?}", e),
            };
        }

        debug!("Acking all messages");

        for (result, msg_ids) in acks {
            match result {
                Ok(batch_result) => {
                    for success in batch_result.successful {
                        (self.on_ack)(self.self_actor.clone().unwrap(), Ok(success.id))
                    }

                    for failure in batch_result.failed {
                        (self.on_ack)(self.self_actor.clone().unwrap(), Err(failure.id))
                    }
                }
                Err(e) => {
                    for msg_id in msg_ids {
                        (self.on_ack)(self.self_actor.clone().unwrap(), Err(msg_id))
                    }
                    warn!("Failed to acknowledge event: {:?}", e);
                }
            }
            // (self.on_ack)(result, message_id);
        }
        debug!("Acked");

        self.completed_events.clear();
        self.completed_messages.clear();

        if let Some(notify) = notify {
            let _ = notify.send(());
        }
    }
}

#[allow(non_camel_case_types)]
pub enum SqsCompletionHandlerMessage<CE, ProcErr, SqsT>
where
    CE: Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    mark_complete {
        msg: SqsMessage,
        completed: OutputEvent<CE, ProcErr>,
    },
    ack_message {
        msg: SqsMessage,
    },
    ack_all {
        notify: Option<tokio::sync::oneshot::Sender<()>>,
    },
    _p {
        _p: std::marker::PhantomData<SqsT>,
    },
}

#[async_trait]
impl<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>
    Actor<SqsCompletionHandlerMessage<CE, ProcErr, SqsT>>
    for SqsCompletionHandler<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
    CPE: Debug + Send + Sync + 'static,
    CP: CompletionEventSerializer<CompletedEvent = CE, Output = Payload, Error = CPE>
        + Send
        + Sync
        + 'static,
    Payload: Send + Sync + 'static,
    CE: Send + Sync + Clone + 'static,
    EE: EventEmitter<Event = Payload> + Send + Sync + 'static,
    OA: Fn(SqsCompletionHandlerActor<CE, ProcErr, SqsT>, Result<String, String>)
        + Send
        + Sync
        + 'static,
    CacheT: Cache + Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self, msg))]
    async fn route_message(&mut self, msg: SqsCompletionHandlerMessage<CE, ProcErr, SqsT>) {
        match msg {
            SqsCompletionHandlerMessage::mark_complete { msg, completed } => {
                self.mark_complete(msg, completed).await
            }
            SqsCompletionHandlerMessage::ack_all { notify } => self.ack_all(notify).await,
            SqsCompletionHandlerMessage::ack_message { msg } => self.ack_message(msg).await,
            SqsCompletionHandlerMessage::_p { .. } => (),
        };
    }

    fn close(&mut self) {
        self.self_actor = None;
    }

    fn get_actor_name(&self) -> &str {
        &self.self_actor.as_ref().unwrap().actor_name
    }
}

pub struct SqsCompletionHandlerActor<CE, ProcErr, SqsT>
where
    CE: Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    sender: Sender<SqsCompletionHandlerMessage<CE, ProcErr, SqsT>>,
    inner_rc: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    queue_len: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    actor_name: String,
    actor_uuid: uuid::Uuid,
    actor_num: u32,
}

impl<CE, ProcErr, SqsT> Clone for SqsCompletionHandlerActor<CE, ProcErr, SqsT>
where
    CE: Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        self.inner_rc
            .clone()
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Self {
            sender: self.sender.clone(),
            inner_rc: self.inner_rc.clone(),
            queue_len: self.queue_len.clone(),
            actor_name: format!(
                "{} {} {}",
                stringify!(SqsCompletionHandlerActor),
                self.actor_uuid,
                self.actor_num + 1,
            ),
            actor_uuid: self.actor_uuid,
            actor_num: self.actor_num + 1,
        }
    }
}

impl<CE, ProcErr, SqsT> SqsCompletionHandlerActor<CE, ProcErr, SqsT>
where
    CE: Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    pub fn new<CPE, CP, Payload, EE, OA, CacheT>(
        mut actor_impl: SqsCompletionHandler<SqsT, CPE, CP, CE, Payload, EE, OA, CacheT, ProcErr>,
    ) -> (Self, tokio::task::JoinHandle<()>)
    where
        SqsT: Sqs + Clone + Send + Sync + 'static,
        CPE: Debug + Send + Sync + 'static,
        CP: CompletionEventSerializer<CompletedEvent = CE, Output = Payload, Error = CPE>
            + Send
            + Sync
            + 'static,
        Payload: Send + Sync + 'static,
        EE: EventEmitter<Event = Payload> + Send + Sync + 'static,
        OA: Fn(SqsCompletionHandlerActor<CE, ProcErr, SqsT>, Result<String, String>)
            + Send
            + Sync
            + 'static,
        CacheT: Cache + Send + Sync + Clone + 'static,
    {
        let (sender, receiver) = channel(1);
        let inner_rc = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(1));

        let queue_len = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let actor_uuid = uuid::Uuid::new_v4();
        let actor_name = format!("{} {} {}", stringify!(#actor_ty), actor_uuid, 0,);
        let self_actor = Self {
            sender,
            inner_rc: inner_rc.clone(),
            queue_len: queue_len.clone(),
            actor_name,
            actor_uuid,
            actor_num: 0,
        };

        actor_impl.self_actor = Some(self_actor.clone());

        let handle = tokio::task::spawn(aktors::actor::route_wrapper(aktors::actor::Router::new(
            actor_impl, receiver, inner_rc, queue_len,
        )));

        (self_actor, handle)
    }

    pub async fn mark_complete(&self, msg: SqsMessage, completed: OutputEvent<CE, ProcErr>) {
        let msg = SqsCompletionHandlerMessage::mark_complete { msg, completed };
        let mut sender = self.sender.clone();

        let queue_len = self.queue_len.clone();
        queue_len.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tokio::task::spawn(async move {
            if let Err(e) = sender.send(msg).await {
                panic!(
                    "Receiver has failed with {}, propagating error. SqsCompletionHandler",
                    e
                )
            }
        });
    }

    pub async fn ack_message(&self, msg: SqsMessage) {
        let msg = SqsCompletionHandlerMessage::ack_message { msg };
        let mut sender = self.sender.clone();

        let queue_len = self.queue_len.clone();
        queue_len.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tokio::task::spawn(async move {
            if let Err(e) = sender.send(msg).await {
                panic!(
                    concat!(
                        "Receiver has failed with {}, propagating error. ",
                        "SqsCompletionHandler"
                    ),
                    e
                )
            }
        });
    }

    async fn ack_all(&self, notify: Option<tokio::sync::oneshot::Sender<()>>) {
        let msg = SqsCompletionHandlerMessage::ack_all { notify };
        let mut sender = self.sender.clone();

        let queue_len = self.queue_len.clone();
        queue_len.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tokio::task::spawn(async move {
            if let Err(e) = sender.send(msg).await {
                panic!(
                    "Receiver has failed with {}, propagating error. SqsCompletionHandler",
                    e
                )
            }
        });
    }

    async fn _p(&self, _p: std::marker::PhantomData<SqsT>) {
        panic!("Invalid to call p");
        let msg = SqsCompletionHandlerMessage::_p { _p };
        if let Err(_e) = self.sender.clone().send(msg).await {
            panic!("Receiver has failed, propagating error. _p")
        }
        self.queue_len
            .clone()
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl<CE, ProcErr, SqsT> Drop for SqsCompletionHandlerActor<CE, ProcErr, SqsT>
where
    CE: Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.inner_rc
            .clone()
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[async_trait]
impl<CE, ProcErr, SqsT> CompletionHandler for SqsCompletionHandlerActor<CE, ProcErr, SqsT>
where
    CE: Send + Sync + Clone + 'static,
    ProcErr: Debug + Send + Sync + 'static,
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    type Message = SqsMessage;
    type CompletedEvent = OutputEvent<CE, ProcErr>;

    async fn mark_complete(&self, msg: Self::Message, completed_event: Self::CompletedEvent) {
        SqsCompletionHandlerActor::mark_complete(self, msg, completed_event).await
    }

    async fn ack_message(&self, msg: Self::Message) {
        SqsCompletionHandlerActor::ack_message(self, msg).await
    }

    async fn ack_all(&self, notify: Option<tokio::sync::oneshot::Sender<()>>) {
        SqsCompletionHandlerActor::ack_all(self, notify).await
    }
}
