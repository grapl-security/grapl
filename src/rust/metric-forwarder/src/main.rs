#![type_length_limit = "1214269"]
// Our types are simply too powerful

mod cloudwatch_logs_parse;
mod cloudwatch_send;
mod deser_logs_data;
mod error;

use aws_lambda_events::event::cloudwatch_logs::CloudwatchLogsEvent;
use lambda_runtime::error::HandlerError;
use lambda_runtime::lambda;
use lambda_runtime::Context;
use log::info;

use crate::cloudwatch_logs_parse::parse_logs;
use crate::cloudwatch_send::statsd_as_cloudwatch_metric_bulk;
use crate::cloudwatch_send::{filter_invalid_stats, get_namespace, put_metric_data};
use crate::error::{to_handler_error, MetricForwarderError};
use rusoto_cloudwatch::CloudWatchClient;
use std::sync::{Arc, Mutex};

fn handler_sync(event: CloudwatchLogsEvent, _ctx: Context) -> Result<(), HandlerError> {
    /**
     * Do some threading magic to block on `handler_async` until it completes
     */
    type R = Result<(), MetricForwarderError>;
    let result_arc: Arc<Mutex<R>> = Arc::new(Mutex::new(Ok(())));
    let result_arc_for_thread = Arc::clone(&result_arc);

    let thread = std::thread::spawn(move || {
        tokio_compat::run_std(async move {
            let result: R = handler_async(event).await.clone();
            *result_arc_for_thread.lock().unwrap() = result;
            ()
        })
    });

    thread.join().unwrap();
    let result = result_arc.lock().unwrap();
    result
        .as_ref()
        .map(|&t| t) // silly conversion from &() to ()
        .map_err(|e| to_handler_error(&e))
}

async fn handler_async(event: CloudwatchLogsEvent) -> Result<(), MetricForwarderError> {
    info!("Handling event");
    let cw_client = get_prod_cloudwatch_client();

    let logs = deser_logs_data::aws_event_to_cloudwatch_logs_data(event);
    match logs {
        Ok(logs) => {
            // Now we have the actual logs.
            let parsed_stats = filter_invalid_stats(parse_logs(logs));
            let namespace = get_namespace(&parsed_stats)?;
            let cloudwatch_metric_data = statsd_as_cloudwatch_metric_bulk(parsed_stats);
            info!("Received {} incoming metrics", cloudwatch_metric_data.len());

            // then forward them to CloudWatch in chunks of 20:
            let put_result = put_metric_data(&cw_client, &cloudwatch_metric_data, &namespace);
            put_result.await
        }
        Err(e) => Err(e),
    }
}

// TODO wimax Sep'20 - need to figure out how to do a local cloudwatch...
fn get_prod_cloudwatch_client() -> CloudWatchClient {
    let region = grapl_config::region();
    CloudWatchClient::new(region)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();

    if env.is_local {
        panic!("yeah, so... this doesn't work locally yet")

    /*
    loop {
        if let Err(e) = local_handler().await {
            error!("local_handler: {}", e);
        };
    }
    */
    } else {
        info!("Running in AWS");
        lambda!(handler_sync);
    }

    Ok(())
}
