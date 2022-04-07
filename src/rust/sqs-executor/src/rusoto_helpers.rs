use std::io::Stdout;

use grapl_observe::{
    metric_reporter::{
        tag,
        MetricReporter,
    },
    timers::{
        time_fut_ms,
        TimedFutureExt,
    },
};
use rusoto_core::RusotoError;
use rusoto_s3::PutObjectError as InnerPutObjectError;
use rusoto_sqs::{
    DeleteMessageError as InnerDeleteMessageError,
    DeleteMessageRequest,
    Message as SqsMessage,
    ReceiveMessageError as InnerReceiveMessageError,
    ReceiveMessageRequest,
    SendMessageRequest,
    Sqs,
};
use tokio::{
    task::{
        JoinError,
        JoinHandle,
    },
    time::error::Elapsed,
};
use tracing::{
    debug,
    error,
    Instrument,
};

use crate::errors::{
    CheckedError,
    Recoverable,
};

impl CheckedError for InnerDeleteMessageError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::InvalidIdFormat(_) => Recoverable::Persistent,
            Self::ReceiptHandleIsInvalid(_) => Recoverable::Persistent,
        }
    }
}

// PutObjectError has no variants, and conveys no information
// about what went wrong, so we must assume a transient error
impl CheckedError for InnerPutObjectError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

impl<E> CheckedError for RusotoError<E>
where
    E: CheckedError + 'static,
{
    /// In all cases, other than Service there's no way to inspect the error
    /// since it's just a String, so we default to assuming it's transient, even though
    /// that may not always be the case.
    fn error_type(&self) -> Recoverable {
        match self {
            RusotoError::Service(e) => e.error_type(),
            RusotoError::HttpDispatch(_)
            | RusotoError::Credentials(_)
            | RusotoError::Validation(_)
            | RusotoError::ParseError(_)
            | RusotoError::Unknown(_)
            | RusotoError::Blocking => Recoverable::Transient,
        }
    }
}

pub async fn get_message<SqsT>(
    queue_url: String,
    sqs_client: SqsT,
    metric_reporter: &mut MetricReporter<Stdout>,
) -> Result<Vec<SqsMessage>, RusotoError<InnerReceiveMessageError>>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    const WAIT_TIME_SECONDS: i64 = 20;
    let messages = sqs_client.receive_message(ReceiveMessageRequest {
        max_number_of_messages: Some(10),
        queue_url,
        visibility_timeout: Some(30),
        wait_time_seconds: Some(WAIT_TIME_SECONDS),
        ..Default::default()
    });

    let messages = tokio::time::timeout(
        std::time::Duration::from_secs((WAIT_TIME_SECONDS as u64) + 1),
        messages,
    );
    let (messages, ms) = time_fut_ms(messages).await;

    let messages = messages
        .expect("timeout")
        .map(|m| m.messages.unwrap_or_else(|| vec![]));

    if let Ok(ref msgs) = messages {
        metric_reporter
            .histogram(
                "sqs_executor.receive_message",
                ms as f64,
                &[tag("success", true), tag("empty_receive", msgs.is_empty())][..],
            )
            .unwrap_or_else(|e| {
                error!(
                    error = e.to_string().as_str(),
                    metric_name = "sqs_executor.receive_message",
                    "Failed to report histogram metric",
                )
            });
    } else {
        metric_reporter
            .histogram(
                "sqs_executor.receive_message",
                ms as f64,
                &[tag("success", false)],
            )
            .unwrap_or_else(|e| {
                error!(
                    error = e.to_string().as_str(),
                    metric_name = "sqs_executor.receive_message",
                    "Failed to report histogram metric",
                )
            });
    };

    messages
}

#[derive(thiserror::Error, Debug)]
pub enum SendMessageError {
    #[error("SendMessageError {0}")]
    InnerSendMessageError(#[from] RusotoError<rusoto_sqs::SendMessageError>),
    #[error("SendMessage Timeout")]
    Timeout(#[from] Elapsed),
}

impl CheckedError for rusoto_sqs::SendMessageError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Persistent
    }
}

impl CheckedError for SendMessageError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::InnerSendMessageError(e) => e.error_type(),
            _ => Recoverable::Transient,
        }
    }
}

pub fn send_message<SqsT>(
    queue_url: String,
    message_body: String,
    sqs_client: SqsT,
    mut metric_reporter: MetricReporter<Stdout>,
) -> JoinHandle<Result<(), SendMessageError>>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    tokio::task::spawn(async move {
        let metric_reporter = &mut metric_reporter;
        let mut last_err = None;
        for i in 0..5u64 {
            let sqs_client = sqs_client.clone();
            let res = sqs_client.send_message(SendMessageRequest {
                queue_url: queue_url.clone(),
                message_body: message_body.clone(),
                ..Default::default()
            });

            let res = tokio::time::timeout(std::time::Duration::from_secs(21), res)
                .timed()
                .await;

            match res {
                (Ok(Ok(_)), ms) => {
                    metric_reporter
                        .histogram(
                            "sqs_executor.send_message.ms",
                            ms as f64,
                            &[tag("success", true)],
                        )
                        .unwrap_or_else(|e| {
                            error!(
                                error = e.to_string().as_str(),
                                metric_name = "sqs_executor.send_message",
                                "Failed to report histogram metric",
                            )
                        });
                    debug!("Send message: {}", queue_url.clone());
                    return Ok(());
                }
                (Ok(Err(e)), ms) => {
                    metric_reporter
                        .histogram(
                            "sqs_executor.send_message.ms",
                            ms as f64,
                            &[tag("success", false)],
                        )
                        .unwrap_or_else(|e| {
                            error!(
                                error = e.to_string().as_str(),
                                metric_name = "sqs_executor.send_message",
                                "Failed to report histogram metric",
                            )
                        });

                    if let Recoverable::Persistent = e.error_type() {
                        return Err(SendMessageError::from(e));
                    } else {
                        last_err = Some(SendMessageError::from(e));
                        tokio::time::sleep(std::time::Duration::from_millis(10 * i)).await;
                    }
                }
                (Err(e), ms) => {
                    metric_reporter
                        .histogram(
                            "sqs_executor.send_message.ms",
                            ms as f64,
                            &[tag("success", false)],
                        )
                        .unwrap_or_else(|e| {
                            error!(
                                error = e.to_string().as_str(),
                                metric_name = "sqs_executor.send_message",
                                "Failed to report histogram metric with timeout",
                            )
                        });

                    last_err = Some(SendMessageError::from(e));
                    tokio::time::sleep(std::time::Duration::from_millis(10 * i)).await;
                }
            }
        }
        Err(last_err.unwrap())
    })
}

#[tracing::instrument(skip(sqs_client, queue_url, receipt_handle, metric_reporter))]
pub fn delete_message<SqsT>(
    sqs_client: SqsT,
    queue_url: String,
    receipt_handle: String,
    mut metric_reporter: MetricReporter<Stdout>,
) -> tokio::task::JoinHandle<()>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    let delete_f = async move {
        let metric_reporter = &mut metric_reporter;
        for _ in 0..5u8 {
            match sqs_client
                .clone()
                .delete_message(DeleteMessageRequest {
                    queue_url: queue_url.clone(),
                    receipt_handle: receipt_handle.clone(),
                })
                .timed()
                .await
            {
                (Ok(_), ms) => {
                    metric_reporter
                        .histogram(
                            "sqs_executor.delete_message.ms",
                            ms as f64,
                            &[tag("success", true)],
                        )
                        .unwrap_or_else(|e| error!("sqs_executor.delete_message.ms: {:?}", e));
                    debug!("Deleted message: {}", receipt_handle.clone());
                    return;
                }
                (Err(e), ms) => {
                    metric_reporter
                        .histogram(
                            "sqs_executor.delete_message.ms",
                            ms as f64,
                            &[tag("success", false)],
                        )
                        .unwrap_or_else(|e| error!("sqs_executor.delete_message.ms: {:?}", e));
                    error!(
                        "Failed to delete_message with: {:?} {:?}",
                        e,
                        e.error_type()
                    );
                    if let Recoverable::Persistent = e.error_type() {
                        return;
                    }
                }
            }
        }
    };
    tokio::task::spawn(delete_f.in_current_span())
}

#[derive(thiserror::Error, Debug)]
pub enum MoveToDeadLetterError {
    #[error("SerializeError {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("SendMessageError {0}")]
    SendMessageError(#[from] SendMessageError),
    #[error("DeleteMessageError {0}")]
    DeleteMessageError(#[from] InnerDeleteMessageError),
    #[error("JoinError {0}")]
    JoinError(#[from] JoinError),
}

pub async fn move_to_dead_letter<SqsT>(
    sqs_client: SqsT,
    message: &impl serde::Serialize,
    publish_to_queue: String,
    delete_from_queue: String,
    receipt_handle: String,
    metric_reporter: MetricReporter<Stdout>,
) -> Result<(), MoveToDeadLetterError>
where
    SqsT: Sqs + Clone + Send + Sync + 'static,
{
    debug!(
        publish_to_queue = publish_to_queue.as_str(),
        delete_from_queue = delete_from_queue.as_str(),
        "Moving message to deadletter queue"
    );
    let message = serde_json::to_string(&message);
    let message = message?;
    send_message(
        publish_to_queue,
        message,
        sqs_client.clone(),
        metric_reporter.clone(),
    )
    .await??;
    delete_message(
        sqs_client,
        delete_from_queue,
        receipt_handle,
        metric_reporter,
    )
    .await?;
    Ok(())
}
