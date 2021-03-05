use std::time::Duration;

use futures::{future::{self,
                       Either},
              pin_mut};
use rusoto_sqs::{ChangeMessageVisibilityRequest,
                 Sqs};
use stopwatch::Stopwatch;
use tokio::sync::{mpsc::{channel as mpsc_channel,
                         Receiver as MpscReceiver,
                         Sender as MpscSender},
                  oneshot::{channel as one_shot,
                            Receiver as OneShotReceiver,
                            Sender as OneShotSender}};
use tracing_futures::Instrument;

struct SqsTimeoutManager<S>
where
    S: Sqs + Send + Sync + 'static,
{
    queue_url: String,
    receipt_handle: String,
    message_id: String,
    visibility_timeout: u64,
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
        tracing::info!("Starting keep_alive for message");

        let sw = Stopwatch::start_new();

        // Sleep for N - 10 seconds, set timeout to 2N
        let message_id = &message_id;
        let max_iter = 10;
        for i in 1..=max_iter {
            let timeout_fut = async {
                tokio::time::sleep(Duration::from_secs(get_next_sleep_for(
                    visibility_timeout,
                    i,
                )))
                .await
            };
            let future_2 = async { receiver.recv().await };
            pin_mut!(timeout_fut);
            pin_mut!(future_2);
            match future::select(timeout_fut, future_2).await {
                Either::Left(_) => {
                    let res = s
                        .change_message_visibility(ChangeMessageVisibilityRequest {
                            queue_url: queue_url.clone(),
                            receipt_handle: receipt_handle.clone(),
                            visibility_timeout: get_next_timeout(visibility_timeout, i),
                        })
                        .await;

                    match res {
                        Ok(()) => {
                            tracing::debug!(
                                iteration = i,
                                message_id = message_id.as_str(),
                                receipt_handle = receipt_handle.as_str(),
                                time_taken = sw.elapsed_ms(),
                                "Successfully changed message visibility"
                            );
                        }
                        Err(rusoto_core::RusotoError::Service(e)) => {
                            tracing::error!(
                                error=?e,
                                iteration=i,
                                message_id=message_id.as_str(),
                                receipt_handle=receipt_handle.as_str(),
                                time_taken=sw.elapsed_ms(),
                                "Failed to change message visibility"
                            );
                            break; // These errors are not retryable
                        }
                        Err(e) => {
                            tracing::warn!(
                                error = e.to_string().as_str(),
                                iteration = i,
                                message_id = message_id.as_str(),
                                receipt_handle = receipt_handle.as_str(),
                                time_taken = sw.elapsed_ms(),
                                "Failed to change message visibility, but it's probably fine"
                            );
                            return;
                        }
                    };
                }
                Either::Right(_) => {
                    tracing::debug!(
                        iteration = i,
                        message_id = message_id.as_str(),
                        receipt_handle = receipt_handle.as_str(),
                        time_taken = sw.elapsed_ms(),
                        "Message no longer needs to be kept alive"
                    );
                    return;
                }
            };

            tracing::debug!(
                iteration = i,
                message_id = message_id.as_str(),
                receipt_handle = receipt_handle.as_str(),
                time_taken = sw.elapsed_ms(),
                "message-visibility-loop",
            );
        }

        tracing::warn!(
            iteration = max_iter,
            message_id = message_id.as_str(),
            receipt_handle = receipt_handle.as_str(),
            time_taken = sw.elapsed_ms(),
            "message still has not processed"
        );
    }
}

fn get_next_sleep_for(initial_timeout: u64, i: u64) -> u64 {
    (initial_timeout * i) - 10
}

fn get_next_timeout(initial_timeout: u64, i: u64) -> i64 {
    (initial_timeout * (i + 1)) as i64
}

// Provides a wrapper for a OneShot to communicate with an Mpsc. This is a hack
// to work around async destructors.
// todo: investigate https://docs.rs/tokio/1.0.1/tokio/sync/mpsc/struct.Sender.html#method.blocking_send
async fn route_oneshot(source_queue: OneShotReceiver<()>, dest_queue: MpscSender<()>) {
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
            let _ = sender.send(()).map_err(|()| {
                tracing::error!("Attempting to drop queue sender, but channel was closed.")
            });
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
    visibility_timeout: u64,
) -> Sender
where
    S: Sqs + Send + Sync + 'static,
{
    let (os_tx, os_rx) = one_shot();
    let (mpsc_tx, mpsc_rx) = mpsc_channel(1);

    let span = tracing::span!(
        tracing::Level::INFO,
        "keep_alive",
        receipt_handle = receipt_handle.as_str(),
        message_id = message_id.as_str(),
        queue_url = queue_url.as_str(),
    );
    let _enter = span.enter();
    let start_f = async move {
        let manager = SqsTimeoutManager {
            queue_url,
            receipt_handle,
            message_id,
            visibility_timeout,
            receiver: mpsc_rx,
            s,
        };
        manager.start().await;
    }
    .in_current_span();
    tokio::task::spawn(start_f);

    let route_f = async move {
        route_oneshot(os_rx, mpsc_tx).await;
    }
    .in_current_span();
    tokio::task::spawn(route_f);

    Sender {
        sender: Some(os_tx),
    }
}
