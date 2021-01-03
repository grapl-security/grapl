use futures::future::{self, Either};
use futures::pin_mut;
use rusoto_sqs::{ChangeMessageVisibilityRequest, Sqs};

use std::time::Duration;

use tokio::stream::StreamExt;
use tokio::sync::mpsc::{channel as mpsc_channel, Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{
    channel as one_shot, Receiver as OneShotReceiver, Sender as OneShotSender,
};

struct SqsTimeoutManager<S>
where
    S: Sqs + Send + Sync + 'static,
{
    queue_url: String,
    receipt_handle: String,
    message_id: String,
    visibility_timeout: i64,
    receiver: MpscReceiver<()>,
    s: S,
}

impl<S> SqsTimeoutManager<S>
where
    S: Sqs + Send + Sync + 'static,
{
    async fn start(self) {
        let Self {
            queue_url,
            receipt_handle,
            message_id,
            visibility_timeout,
            mut receiver,
            s,
        } = self;

        // let mut sw = Stopwatch::start_new();
        // every N / 2 seconds, set the visibility timeout to N * 2, set N to N * 2
        // Basically, we double the timeout every time, and update it halfway through
        // We could be a bit smarter about it though if we wanted to and update it more than halfway
        // but whatever
        // todo: Handle errors - we can retry some of the errors
        // todo: Metrics
        // if we ever go beyond a deadline we should be sure to
        let mut last_timeout = visibility_timeout;
        let message_id = &message_id;
        for i in 1..100 {
            let timeout_fut = async {
                tokio::time::delay_for(Duration::from_secs(((last_timeout * i) as u64) / 2)).await
            };
            let future_2 = async { receiver.recv().await };
            pin_mut!(timeout_fut);
            pin_mut!(future_2);
            // wait for N / 2 seconds or a message to stop
            match future::select(timeout_fut, future_2).await {
                Either::Left(_) => {
                    let res = s
                        .change_message_visibility(ChangeMessageVisibilityRequest {
                            queue_url: queue_url.clone(),
                            receipt_handle: receipt_handle.clone(),
                            visibility_timeout: visibility_timeout * (i + 1),
                        })
                        .await;

                    match res {
                        Ok(()) => {
                            tracing::debug!("Updated message visibility: {} {} {}", i, message_id, &receipt_handle);
                        }
                        Err(rusoto_core::RusotoError::Service(e)) => {
                            tracing::error!("Failed to change message visibility: {} {} {}", i, message_id, &receipt_handle);
                            break; // These errors are not retryable
                        }
                        Err(e) => {
                            tracing::error!("Failed to change message visibility: {} {} {}", i, message_id, &receipt_handle);
                        }
                    };
                }
                Either::Right(_) => break,
            };

            tracing::debug!("message-visibility-loop: {} {} {}", i, message_id, &receipt_handle);
        }

        tracing::warn!("message still has not processed after 100 iterators {} {}", message_id, &receipt_handle);
        // let elapsed = sw.elapsed_ms();
    }
}

// Provides a wrapper for a OneShot to communicate with an Mpsc. This is a hack
// to work around async destructors.
// todo: investigate https://docs.rs/tokio/1.0.1/tokio/sync/mpsc/struct.Sender.html#method.blocking_send
async fn route_oneshot(source_queue: OneShotReceiver<()>, mut dest_queue: MpscSender<()>) {
    let _ = source_queue.await;
    let _ = dest_queue.send(()).await;
}

// Do not impl Clone on this. Use an Arc if you need to clone.
/// Sends a message one time, either explicitly via `stop` or implicitly via `Drop`
pub struct Sender {
    sender: Option<OneShotSender<()>>,
}

impl Sender {
    pub fn stop(self) {}
}

impl Drop for Sender {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(());
        }
    }
}

/// Given a message receipt, a queue, and a timeout, `keep_alive` will ensure
/// that a message will stay alive during the lifetime of the returned `Sender`.
pub fn keep_alive<S>(
    s: S,
    receipt_handle: String,
    message_id: String,
    queue_url: String,
    visibility_timeout: i64,
) -> Sender
where
    S: Sqs + Send + Sync + 'static,
{
    let (os_tx, os_rx) = one_shot();
    let (mpsc_tx, mpsc_rx) = mpsc_channel(1);

    tokio::task::spawn(async move {
        let manager = SqsTimeoutManager {
            queue_url,
            receipt_handle,
            message_id,
            visibility_timeout,
            receiver: mpsc_rx,
            s,
        };
        manager.start().await;
    });

    tokio::task::spawn(async move {
        route_oneshot(os_rx, mpsc_tx).await;
    });

    Sender {
        sender: Some(os_tx),
    }
}
